/**
 * 类型安全的环境变量访问层。
 * 对齐 kanban 现有约定：使用 VITE_API_BASE（相对路径，dev 下走 vite proxy /api）。
 * 与模板的 zod 严格校验不同：此处采用非抛错策略，缺失时回退默认值，避免本期引入硬依赖。
 */
const env = {
  get VITE_API_BASE() {
    return (import.meta.env.VITE_API_BASE as string | undefined) ?? "";
  },
} as const;

export default env;
