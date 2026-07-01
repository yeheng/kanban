/**
 * 后端响应封装类型（对齐模板）。
 * kanban 后端目前直返裸数据（如 Project[]），故 IResponse 本期为可选工具类型，
 * composable 暂不强制用它包裹。将来后端统一响应格式时启用。
 */
export interface IResponse<T, E = Record<string, unknown>> {
  data: T;
  extra: E;
  code: number;
  message: string;
  success: boolean;
}

export interface IPaginationRequestQuery {
  page?: number;
  pageSize?: number;
}

export type IRequestQuery<T extends Record<string, unknown>> = {
  page?: number;
  pageSize?: number;
} & {
  [K in keyof T]?: T[K];
};
