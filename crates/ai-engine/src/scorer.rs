use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait Scorer: Send + Sync {
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64; // 0..1
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
    async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
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

/// Production semantic scorer via `rig` embeddings (local Ollama default, design §5).
/// Cosine similarity over embeddings of resource skill/tag text vs task requirement text.
/// Mandatory skills unmet ⇒ 0 (consistent with FallbackScorer). Returns 0.0 on any provider
/// error so the engine degrades gracefully (confirmed #8 degradation).
#[cfg(feature = "llm")]
pub mod semantic {
    use crate::types::*;
    use async_trait::async_trait;
    use rig::client::EmbeddingsClient;
    use rig::client::ProviderClient;
    use rig::embeddings::EmbeddingModel;
    use rig::providers::ollama;

    pub struct SemanticScorer {
        pub model: String,
        pub base_url: Option<String>,
    }

    impl SemanticScorer {
        fn client(&self) -> Option<ollama::Client> {
            // from_env reads OLLAMA_API_BASE_URL (default http://localhost:11434). An explicit
            // base_url override would use the builder; from_env is the documented config knob.
            let _ = &self.base_url; // reserved for a future builder() override
            ollama::Client::from_env().ok()
        }
    }

    #[async_trait]
    impl super::Scorer for SemanticScorer {
        async fn score(&self, r: &CandidateResource, t: &CandidateTask) -> f64 {
            // Hard filter: mandatory skills must be met (same rule as FallbackScorer).
            for req in &t.skill_reqs {
                if req.is_mandatory {
                    if !matches!(r.skills.get(&req.skill_id), Some(p) if *p >= req.min_proficiency)
                    {
                        return 0.0;
                    }
                }
            }
            let Some(client) = self.client() else {
                return 0.0;
            };
            let model = client.embedding_model(&self.model);
            let r_text = format!("skills={:?} tags={:?}", r.skills, r.tags);
            let t_text = format!("{} reqs={:?}", t.title, t.skill_reqs);
            let er = match model.embed_text(&r_text).await {
                Ok(e) => e,
                Err(_) => return 0.0,
            };
            let et = match model.embed_text(&t_text).await {
                Ok(e) => e,
                Err(_) => return 0.0,
            };
            cosine(&er.vec, &et.vec).max(0.0)
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
        #[ignore = "needs Ollama running with the embed model"]
        async fn smoke_semantic() {
            let s = SemanticScorer {
                model: "nomic-embed-text".into(),
                base_url: None,
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
