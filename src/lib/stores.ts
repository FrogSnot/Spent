import { writable, derived } from 'svelte/store';

export interface CurrencySettings {
  code: string;
  symbol: string;
  position: 'before' | 'after';
  locale: string;
}

const defaultCurrency: CurrencySettings = {
  code: 'USD',
  symbol: '$',
  position: 'before',
  locale: 'en-US'
};

function loadCurrency(): CurrencySettings {
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('spent_currency');
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch (e) {
        console.error('Failed to parse stored currency:', e);
      }
    }
  }
  return defaultCurrency;
}

export const currencySettings = writable<CurrencySettings>(loadCurrency());

currencySettings.subscribe(value => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('spent_currency', JSON.stringify(value));
  }
});

export interface CurrencyOption {
  code: string;
  symbol: string;
  name: string;
  position: 'before' | 'after';
  locale: string;
  custom?: boolean;
}

export const currencyOptions: CurrencyOption[] = [
  { code: 'USD', symbol: '$', name: 'US Dollar', position: 'before', locale: 'en-US' },
  { code: 'EUR', symbol: '€', name: 'Euro', position: 'after', locale: 'de-DE' },
  { code: 'GBP', symbol: '£', name: 'British Pound', position: 'before', locale: 'en-GB' },
  { code: 'JPY', symbol: '¥', name: 'Japanese Yen', position: 'before', locale: 'ja-JP' },
  { code: 'CAD', symbol: 'CA$', name: 'Canadian Dollar', position: 'before', locale: 'en-CA' },
  { code: 'AUD', symbol: 'A$', name: 'Australian Dollar', position: 'before', locale: 'en-AU' },
  { code: 'CHF', symbol: 'CHF', name: 'Swiss Franc', position: 'before', locale: 'de-CH' },
  { code: 'CNY', symbol: '¥', name: 'Chinese Yuan', position: 'before', locale: 'zh-CN' },
  { code: 'INR', symbol: '₹', name: 'Indian Rupee', position: 'before', locale: 'en-IN' },
  { code: 'BRL', symbol: 'R$', name: 'Brazilian Real', position: 'before', locale: 'pt-BR' },
  { code: 'MXN', symbol: 'MX$', name: 'Mexican Peso', position: 'before', locale: 'es-MX' },
  { code: 'ZAR', symbol: 'R', name: 'South African Rand', position: 'before', locale: 'en-ZA' },
  { code: 'KRW', symbol: '₩', name: 'South Korean Won', position: 'before', locale: 'ko-KR' },
  { code: 'SEK', symbol: 'kr', name: 'Swedish Krona', position: 'after', locale: 'sv-SE' },
  { code: 'NOK', symbol: 'kr', name: 'Norwegian Krone', position: 'after', locale: 'nb-NO' },
  { code: 'DKK', symbol: 'kr', name: 'Danish Krone', position: 'after', locale: 'da-DK' },
  { code: 'PLN', symbol: 'zł', name: 'Polish Złoty', position: 'after', locale: 'pl-PL' },
  { code: 'RUB', symbol: '₽', name: 'Russian Ruble', position: 'after', locale: 'ru-RU' },
  { code: 'SGD', symbol: 'S$', name: 'Singapore Dollar', position: 'before', locale: 'en-SG' },
  { code: 'HKD', symbol: 'HK$', name: 'Hong Kong Dollar', position: 'before', locale: 'zh-HK' },
  { code: 'NZD', symbol: 'NZ$', name: 'New Zealand Dollar', position: 'before', locale: 'en-NZ' },
  { code: 'TRY', symbol: '₺', name: 'Turkish Lira', position: 'before', locale: 'tr-TR' },
];

function loadCustomCurrencies(): CurrencyOption[] {
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('spent_custom_currencies');
    if (stored) {
      try {
        return JSON.parse(stored);
      } catch (e) {
        console.error('Failed to parse custom currencies:', e);
      }
    }
  }
  return [];
}

export const customCurrencies = writable<CurrencyOption[]>(loadCustomCurrencies());

customCurrencies.subscribe(value => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('spent_custom_currencies', JSON.stringify(value));
  }
});

export const allCurrencyOptions = derived(customCurrencies, $custom => [
  ...currencyOptions,
  ...$custom,
]);

export function formatCurrency(cents: number, settings: CurrencySettings): string {
  const dollars = Math.abs(cents) / 100;
  const sign = cents < 0 ? '−' : '';
  
  const formatted = new Intl.NumberFormat(settings.locale, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(dollars);
  
  if (settings.position === 'before') {
    return `${sign}${settings.symbol}${formatted}`;
  } else {
    return `${sign}${formatted} ${settings.symbol}`;
  }
}

export interface AppSettings {
  dateFormat: 'MM/DD/YYYY' | 'DD/MM/YYYY' | 'YYYY-MM-DD';
  weekStart: 'sunday' | 'monday';
  transactionLimit: number;
  defaultCategory: string;
  confirmBeforeDelete: boolean;
}

const defaultAppSettings: AppSettings = {
  dateFormat: 'MM/DD/YYYY',
  weekStart: 'sunday',
  transactionLimit: 50,
  defaultCategory: 'Other',
  confirmBeforeDelete: true,
};

function loadAppSettings(): AppSettings {
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem('spent_app_settings');
    if (stored) {
      try {
        return { ...defaultAppSettings, ...JSON.parse(stored) };
      } catch (e) {
        console.error('Failed to parse app settings:', e);
      }
    }
  }
  return defaultAppSettings;
}

export const appSettings = writable<AppSettings>(loadAppSettings());

appSettings.subscribe(value => {
  if (typeof window !== 'undefined') {
    localStorage.setItem('spent_app_settings', JSON.stringify(value));
  }
});
