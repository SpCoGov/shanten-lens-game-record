export type PluginApi = {
    i18n: {locale(): string; t(key: string, values?: Record<string, unknown>): string; onChange(callback: () => void): () => void};
    backend: {
        invoke<T>(method: string, params?: unknown): Promise<T>;
        request<T>(moduleId: string, method: string, payload: unknown): Promise<T>;
        onEvent(name: string, callback: (data: unknown) => void): () => void;
    };
    ui: {
        registerPage(page: {id: string; title: () => string; icon: string; mount(container: HTMLElement): () => void}): () => void;
        openPage(id: string): void;
        showToast(message: string, kind?: "info" | "success" | "error"): void;
    };
};

let host: PluginApi | undefined;
export function setHost(api: PluginApi | undefined) { host = api; }
export function getHost(): PluginApi {
    if (!host) throw new Error("plugin is inactive");
    return host;
}
