import React from "react";
import {getHost} from "./host";

export function useTranslation() {
    const api = getHost();
    const locale = React.useSyncExternalStore(api.i18n.onChange, api.i18n.locale, api.i18n.locale);
    const t = React.useCallback((key: string, values?: Record<string, unknown>) => api.i18n.t(key, values), [api, locale]);
    return {t, i18n: {language: locale, resolvedLanguage: locale}};
}
