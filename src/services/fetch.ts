import { ofetch } from "ofetch";
import env from "@/utils/env";

/**
 * ofetch 客户端。baseURL 来自 @/utils/env（相对路径，dev 走 vite proxy /api）。
 * 拦截器：onRequest 预留 auth header（kanban 无 auth，本期空）；
 * onResponseError 统一打日志（注意：HTTP 状态错误时 context.error 未设置，
 * 有意义的信息在 context.response —— status 与 _data）。
 */
const apiFetch = ofetch.create({
  baseURL: env.VITE_API_BASE,
  onRequest(_ctx) {
    // 预留：auth header 注入（kanban 当前无 auth）
  },
  onResponseError(ctx) {
    const status = ctx.response?.status;
    const body = ctx.response?._data;
    console.error("[api] request failed:", status, body);
  },
});

export function useApiFetch() {
  return { apiFetch };
}
