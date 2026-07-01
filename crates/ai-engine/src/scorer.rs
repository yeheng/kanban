use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait Scorer: Send + Sync {
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64; // 0..1
    #[tracing::instrument(skip(self, problem), fields(resources = problem.resources.len(), tasks = problem.tasks.len()))]
    async fn matrix(&self, problem: &AllocationProblem) -> ScoreMatrix {
        let mut m = ScoreMatrix::new();
        for r in &problem.resources {
            for t in &problem.tasks {
                m.insert((r.id, t.id), self.score(r, t).await);
            }
        }
        m
    }
}

/// Deterministic offline scorer (no LLM). Keyword-Jaccard over resource
/// skills+tags vs task title+skill_reqs (coarse proficiency buckets), plus a
/// proficiency bonus. Mandatory skills unmet ⇒ 0 (hard filter reflected in score).
/// Weights are configurable (default 0.6/0.4 jaccard/proficiency).
pub struct FallbackScorer {
    pub w_jaccard: f64,
    pub w_proficiency: f64,
}

impl Default for FallbackScorer {
    fn default() -> Self {
        Self { w_jaccard: 0.6, w_proficiency: 0.4 }
    }
}

impl FallbackScorer {
    fn tokens(r: &CandidateResource) -> Vec<String> {
        let mut v: Vec<String> = r.tags.to_vec();
        for (sid, prof) in &r.skills {
            v.push(format!("skill{}p{}", sid, prof / 3));
        } // coarse bucket
        v.into_iter().map(|s| s.to_lowercase()).collect()
    }
    fn task_tokens(t: &CandidateTask) -> Vec<String> {
        let mut v: Vec<String> = t
            .title
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        for req in &t.skill_reqs {
            v.push(format!("skill{}p{}", req.skill_id, req.min_proficiency / 3));
        }
        v
    }
    fn jaccard(a: &[String], b: &[String]) -> f64 {
        let sa: std::collections::HashSet<&String> = a.iter().collect();
        let sb: std::collections::HashSet<&String> = b.iter().collect();
        if sa.is_empty() && sb.is_empty() {
            return 0.0;
        }
        let inter = sa.intersection(&sb).count() as f64;
        let union = sa.union(&sb).count() as f64;
        inter / union
    }
}

#[async_trait]
impl Scorer for FallbackScorer {
    #[tracing::instrument(skip(self, r, t), fields(resource_id = r.id, task_id = t.id))]
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
        self.score_sync(r, t)
    }
    /// Pure-CPU scorer — build the matrix without R·T awaits (default impl would).
    #[tracing::instrument(skip(self, problem), fields(resources = problem.resources.len(), tasks = problem.tasks.len()))]
    async fn matrix(&self, problem: &AllocationProblem) -> ScoreMatrix {
        tracing::debug!("building fallback score matrix");
        let mut m = ScoreMatrix::with_capacity(problem.resources.len() * problem.tasks.len());
        for r in &problem.resources {
            for t in &problem.tasks {
                m.insert((r.id, t.id), self.score_sync(r, t));
            }
        }
        m
    }
}

impl FallbackScorer {
    fn score_sync(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
        // mandatory skills must be met at min proficiency, else 0 (hard filter reflected in score)
        for req in &t.skill_reqs {
            if req.is_mandatory {
                match r.skills.get(&req.skill_id) {
                    Some(p) if *p >= req.min_proficiency => {}
                    _ => return 0.0,
                }
            }
        }
        let base = Self::jaccard(&Self::tokens(r), &Self::task_tokens(t));
        // proficiency bonus: avg proficiency on required skills / 5
        let bonus = if t.skill_reqs.is_empty() {
            0.0
        } else {
            let s: f64 = t
                .skill_reqs
                .iter()
                .filter_map(|req| r.skills.get(&req.skill_id))
                .map(|p| *p as f64)
                .sum();
            s / (t.skill_reqs.len() as f64 * 5.0)
        };
        (base * self.w_jaccard + bonus * self.w_proficiency).clamp(0.0, 1.0)
    }
}

/// Production semantic scorer via `rig` embeddings. Cosine similarity over embeddings
/// of resource skill/tag text vs task requirement text. Supports Ollama and OpenAI
/// embedding providers; any unsupported provider or configuration error degrades to 0.0
/// so the engine never panics (design §2.8 graceful degradation).
#[cfg(feature = "llm")]
pub mod semantic {
    use crate::llm_client::{embed_text, LlmClientConfig};
    use crate::types::*;
    use async_trait::async_trait;

    pub struct SemanticScorer {
        pub provider: String,
        pub model: String,
        pub base_url: Option<String>,
        pub api_key: Option<String>,
        /// Fallback scorer used when the embedding provider is unreachable or returns no
        /// embeddings, so the optimization pipeline still produces deterministic scores.
        pub fallback: super::FallbackScorer,
    }

    fn cfg(s: &SemanticScorer) -> LlmClientConfig {
        LlmClientConfig {
            provider: s.provider.clone(),
            base_url: s.base_url.clone(),
            api_key: s.api_key.clone(),
            model: s.model.clone(),
        }
    }

    fn mandatory_skills_met(r: &CandidateResource, t: &CandidateTask) -> bool {
        for req in &t.skill_reqs {
            if req.is_mandatory
                && !matches!(r.skills.get(&req.skill_id), Some(p) if *p >= req.min_proficiency)
            {
                return false;
            }
        }
        true
    }

    fn resource_text(r: &CandidateResource) -> String {
        format!("skills={:?} tags={:?}", r.skills, r.tags)
    }
    fn task_text(t: &CandidateTask) -> String {
        format!("{} reqs={:?}", t.title, t.skill_reqs)
    }

    #[async_trait]
    impl super::Scorer for SemanticScorer {
        #[tracing::instrument(skip(self, r, t), fields(resource_id = r.id, task_id = t.id, provider = %self.provider, model = %self.model))]
        async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
            if !mandatory_skills_met(r, t) {
                return 0.0;
            }
            let cfg = cfg(self);
            let er = match embed_text(&cfg, &resource_text(r)).await {
                Some(e) => e,
                None => return 0.0,
            };
            let et = match embed_text(&cfg, &task_text(t)).await {
                Some(e) => e,
                None => return 0.0,
            };
            cosine(&er, &et).max(0.0)
        }

        /// Pre-embed each resource and each task ONCE (R+T embeds), then cosine every
        /// pair. The default impl would call `score` R·T times and re-embed resources
        /// for every task — O(R·T) round-trips instead of O(R+T). If the provider is
        /// unreachable or returns no embeddings, fall back to the deterministic
        /// `FallbackScorer` so the optimization pipeline still produces usable scores
        /// instead of an all-zero matrix (design §2.8 graceful degradation).
        #[tracing::instrument(skip(self, problem), fields(resources = problem.resources.len(), tasks = problem.tasks.len(), provider = %self.provider, model = %self.model))]
        async fn matrix(&self, problem: &AllocationProblem) -> ScoreMatrix {
            use std::collections::HashMap;
            let mut m = ScoreMatrix::with_capacity(problem.resources.len() * problem.tasks.len());
            let cfg = cfg(self);

            tracing::debug!("building semantic score matrix");
            let mut er: HashMap<i64, Vec<f64>> = HashMap::new();
            for r in &problem.resources {
                if let Some(e) = embed_text(&cfg, &resource_text(r)).await {
                    er.insert(r.id, e);
                }
            }
            let mut et: HashMap<i64, Vec<f64>> = HashMap::new();
            for t in &problem.tasks {
                if let Some(e) = embed_text(&cfg, &task_text(t)).await {
                    et.insert(t.id, e);
                }
            }

            let provider_failed = (problem.resources.is_empty() || er.is_empty())
                && (problem.tasks.is_empty() || et.is_empty());
            if provider_failed {
                tracing::warn!(
                    provider = %self.provider,
                    "semantic provider returned no embeddings; falling back to FallbackScorer"
                );
                return self.fallback.matrix(problem).await;
            }

            for r in &problem.resources {
                let Some(vr) = er.get(&r.id) else { continue };
                for t in &problem.tasks {
                    let Some(vt) = et.get(&t.id) else { continue };
                    let sc = if mandatory_skills_met(r, t) {
                        cosine(vr, vt).max(0.0)
                    } else {
                        0.0
                    };
                    m.insert((r.id, t.id), sc);
                }
            }
            m
        }
    }

    fn cosine(a: &[f64], b: &[f64]) -> f64 {
        let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        if na == 0.0 || nb == 0.0 {
            0.0
        } else {
            dot / (na * nb)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::scorer::Scorer;
        #[tokio::test]
        #[ignore = "needs LLM provider running with the embed model"]
        async fn smoke_semantic() {
            let s = SemanticScorer {
                provider: "ollama".into(),
                model: "nomic-embed-text".into(),
                base_url: None,
                api_key: None,
                fallback: super::super::FallbackScorer::default(),
            };
            let _ = s
                .score(
                    &CandidateResource {
                        id: 1,
                        name: "R".into(),
                        skills: Default::default(),
                        tags: vec!["rust".into()],
                        daily_capacity_pd: 1.0,
                        available_from: None,
                        available_to: None,
                    },
                    &CandidateTask {
                        id: 1,
                        project_id: 1,
                        title: "rust backend".into(),
                        estimate_pd: 1.0,
                        start: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
                        end: chrono::NaiveDate::from_ymd_opt(2026, 7, 2).unwrap(),
                        priority: 1,
                        skill_reqs: vec![],
                    },
                )
                .await;
        }
    }
}
