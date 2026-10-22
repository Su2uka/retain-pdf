export type QueryValue = string | number | boolean | null | undefined | ReadonlyArray<string | number>;
export interface RequestOptions {
    signal?: AbortSignal;
    /** 把失败的响应变成错误对象；不传时用后端的 message 加状态码，并挂上 errorCode、reason。 */
    onError?: (status: number, payload: any) => Error;
}
export interface RequestSpec {
    apiPrefix: string | undefined;
    method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
    /** 相对 /api/v1 的路径，路径参数已经编码。 */
    path: string;
    query?: object;
    body?: unknown;
    options?: RequestOptions;
}
/** 查询参数按传入顺序编码：undefined、null、空串不发；数组按后端约定用逗号连接（job_ids=a,b）。 */
export declare function encodeQuery(query: object | undefined): string;
export declare function encodePathParam(value: string): string;
export declare function request<T>({ apiPrefix, method, path, query, body, options }: RequestSpec): Promise<T>;
/** 只用固定文案加状态码，例如「读取文档库失败，请稍后重试。(500)」。 */
export declare function failWith(message: string): RequestOptions["onError"];
/** 优先用后端回的 message，没有时用固定文案，都加状态码。 */
export declare function failWithServerMessage(fallback: string): RequestOptions["onError"];
