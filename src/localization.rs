use std::{
    collections::{BTreeMap, BTreeSet},
    fmt, fs,
    path::Path,
};

use fluent_bundle::{
    concurrent::FluentBundle as ConcurrentFluentBundle, FluentArgs, FluentResource,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use unic_langid::{CharacterDirection, LanguageIdentifier};

use crate::{ZsuiError, ZsuiResult};

type FluentBundle = ConcurrentFluentBundle<FluentResource>;

/// A normalized Unicode language identifier used for resource lookup and UI
/// direction. Underscores are accepted at input boundaries and normalized to
/// BCP 47-style hyphens.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ZsLocale(LanguageIdentifier);

impl ZsLocale {
    pub fn parse(tag: impl AsRef<str>) -> ZsuiResult<Self> {
        let raw = tag.as_ref().trim();
        let normalized = raw.replace('_', "-");
        normalized
            .parse::<LanguageIdentifier>()
            .map(Self)
            .map_err(|err| {
                ZsuiError::invalid_spec("locale", format!("invalid language tag `{raw}`: {err}"))
            })
    }

    /// Returns the first valid locale reported by the operating system.
    pub fn detect() -> Option<Self> {
        sys_locale::get_locales().find_map(|raw| {
            let tag = raw.split('.').next().unwrap_or(&raw);
            let tag = tag.split('@').next().unwrap_or(tag);
            Self::parse(tag).ok()
        })
    }

    pub fn language(&self) -> &str {
        self.0.language.as_str()
    }

    pub fn direction(&self) -> ZsTextDirection {
        match self.0.character_direction() {
            CharacterDirection::RTL => ZsTextDirection::RightToLeft,
            CharacterDirection::LTR => ZsTextDirection::LeftToRight,
            CharacterDirection::TTB => ZsTextDirection::TopToBottom,
        }
    }

    /// Produces the deterministic resource lookup chain for this locale.
    /// `zh-Hant-TW` resolves through `zh-Hant-TW`, `zh-Hant`, then `zh`.
    pub fn fallback_chain(&self) -> Vec<Self> {
        let mut chain = Vec::new();
        push_unique_locale(&mut chain, self.0.clone());

        if self.0.variants().len() > 0 {
            let mut without_variants = self.0.clone();
            without_variants.clear_variants();
            push_unique_locale(&mut chain, without_variants);
        }

        if self.0.region.is_some() {
            let mut without_region = self.0.clone();
            without_region.clear_variants();
            without_region.region = None;
            push_unique_locale(&mut chain, without_region);
        }

        if self.0.script.is_some() || self.0.region.is_some() {
            let mut language_only = self.0.clone();
            language_only.clear_variants();
            language_only.script = None;
            language_only.region = None;
            push_unique_locale(&mut chain, language_only);
        }

        chain
    }
}

impl fmt::Display for ZsLocale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ZsLocale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ZsLocale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tag = String::deserialize(deserializer)?;
        Self::parse(&tag).map_err(serde::de::Error::custom)
    }
}

fn push_unique_locale(chain: &mut Vec<ZsLocale>, locale: LanguageIdentifier) {
    let locale = ZsLocale(locale);
    if !chain.contains(&locale) {
        chain.push(locale);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ZsTextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ZsMessageValue {
    Text(String),
    Number(f64),
}

impl From<String> for ZsMessageValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for ZsMessageValue {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

macro_rules! message_number_from {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl From<$ty> for ZsMessageValue {
                fn from(value: $ty) -> Self {
                    Self::Number(value as f64)
                }
            }
        )+
    };
}

message_number_from!(u8, u16, u32, i8, i16, i32, f32, f64);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ZsMessageArgs {
    values: BTreeMap<String, ZsMessageValue>,
}

impl ZsMessageArgs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<ZsMessageValue>) -> Self {
        self.insert(name, value);
        self
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<ZsMessageValue>) {
        self.values.insert(name.into(), value.into());
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

/// Application-owned localization state. Catalog registration and locale
/// changes are explicit; no process-global mutable translation registry is
/// created. A locale change can therefore participate in the application's
/// normal typed update and View rebuild path.
pub struct ZsLocalizer {
    locale: ZsLocale,
    fallback_locale: ZsLocale,
    bundles: BTreeMap<ZsLocale, FluentBundle>,
}

impl fmt::Debug for ZsLocalizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ZsLocalizer")
            .field("locale", &self.locale)
            .field("fallback_locale", &self.fallback_locale)
            .field("resource_locales", &self.bundles.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl ZsLocalizer {
    pub fn new(locale: ZsLocale, fallback_locale: ZsLocale) -> Self {
        Self {
            locale,
            fallback_locale,
            bundles: BTreeMap::new(),
        }
    }

    pub fn for_system(fallback_locale: ZsLocale) -> Self {
        let locale = ZsLocale::detect().unwrap_or_else(|| fallback_locale.clone());
        Self::new(locale, fallback_locale)
    }

    pub fn locale(&self) -> &ZsLocale {
        &self.locale
    }

    pub fn fallback_locale(&self) -> &ZsLocale {
        &self.fallback_locale
    }

    pub fn direction(&self) -> ZsTextDirection {
        self.locale.direction()
    }

    pub fn set_locale(&mut self, locale: ZsLocale) {
        self.locale = locale;
    }

    pub fn set_locale_tag(&mut self, tag: impl AsRef<str>) -> ZsuiResult<()> {
        self.set_locale(ZsLocale::parse(tag)?);
        Ok(())
    }

    pub fn add_ftl(&mut self, locale: ZsLocale, source: impl Into<String>) -> ZsuiResult<()> {
        let resource = parse_ftl(&locale, source.into())?;
        let bundle = self.bundle_for(locale);
        bundle.add_resource(resource).map_err(|errors| {
            ZsuiError::invalid_spec(
                "localization.resource",
                format!("duplicate or invalid Fluent message: {errors:?}"),
            )
        })
    }

    pub fn add_ftl_overriding(
        &mut self,
        locale: ZsLocale,
        source: impl Into<String>,
    ) -> ZsuiResult<()> {
        let resource = parse_ftl(&locale, source.into())?;
        self.bundle_for(locale).add_resource_overriding(resource);
        Ok(())
    }

    pub fn add_ftl_file(&mut self, locale: ZsLocale, path: impl AsRef<Path>) -> ZsuiResult<()> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|err| {
            ZsuiError::host(
                "load localization resource",
                format!("{}: {err}", path.display()),
            )
        })?;
        self.add_ftl(locale, source)
    }

    pub fn has_message(&self, id: &str) -> bool {
        self.lookup_locales()
            .iter()
            .filter_map(|locale| self.bundles.get(locale))
            .any(|bundle| bundle.has_message(id))
    }

    pub fn text(&self, id: &str, fallback: impl Into<String>) -> String {
        self.format(id, &ZsMessageArgs::default(), fallback)
    }

    pub fn format(&self, id: &str, args: &ZsMessageArgs, fallback: impl Into<String>) -> String {
        self.try_format(id, args)
            .ok()
            .flatten()
            .unwrap_or_else(|| fallback.into())
    }

    /// Resolves a message through the active locale, its parents and the
    /// configured fallback locale. Missing messages return `Ok(None)`; invalid
    /// message formatting returns an error so catalog problems remain visible.
    pub fn try_format(&self, id: &str, args: &ZsMessageArgs) -> ZsuiResult<Option<String>> {
        let mut fluent_args = FluentArgs::new();
        for (name, value) in &args.values {
            match value {
                ZsMessageValue::Text(value) => fluent_args.set(name, value.as_str()),
                ZsMessageValue::Number(value) => fluent_args.set(name, *value),
            }
        }
        let fluent_args = (!args.is_empty()).then_some(&fluent_args);

        for locale in self.lookup_locales() {
            let Some(bundle) = self.bundles.get(&locale) else {
                continue;
            };
            let Some(message) = bundle.get_message(id) else {
                continue;
            };
            let Some(pattern) = message.value() else {
                continue;
            };
            let mut errors = Vec::new();
            let text = bundle
                .format_pattern(pattern, fluent_args, &mut errors)
                .into_owned();
            if errors.is_empty() {
                return Ok(Some(text));
            }
            return Err(ZsuiError::invalid_spec(
                "localization.message",
                format!("failed to format `{id}` for `{locale}`: {errors:?}"),
            ));
        }

        Ok(None)
    }

    fn bundle_for(&mut self, locale: ZsLocale) -> &mut FluentBundle {
        self.bundles.entry(locale.clone()).or_insert_with(|| {
            let mut bundle = FluentBundle::new_concurrent(vec![locale.0]);
            bundle.set_use_isolating(true);
            bundle
        })
    }

    fn lookup_locales(&self) -> Vec<ZsLocale> {
        let mut seen = BTreeSet::new();
        self.locale
            .fallback_chain()
            .into_iter()
            .chain(self.fallback_locale.fallback_chain())
            .filter(|locale| seen.insert(locale.clone()))
            .collect()
    }
}

fn parse_ftl(locale: &ZsLocale, source: String) -> ZsuiResult<FluentResource> {
    FluentResource::try_new(source).map_err(|(_, errors)| {
        ZsuiError::invalid_spec(
            "localization.resource",
            format!("invalid Fluent resource for `{locale}`: {errors:?}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn locale(tag: &str) -> ZsLocale {
        ZsLocale::parse(tag).expect("test locale should be valid")
    }

    #[test]
    fn locale_normalizes_and_builds_script_region_fallbacks() {
        let locale = locale("ZH_hant_tw");
        assert_eq!(locale.to_string(), "zh-Hant-TW");
        assert_eq!(
            locale
                .fallback_chain()
                .into_iter()
                .map(|locale| locale.to_string())
                .collect::<Vec<_>>(),
            vec!["zh-Hant-TW", "zh-Hant", "zh"]
        );
    }

    #[test]
    fn locale_reports_ui_direction() {
        assert_eq!(locale("en-US").direction(), ZsTextDirection::LeftToRight);
        assert_eq!(locale("ar").direction(), ZsTextDirection::RightToLeft);
    }

    #[test]
    fn localizer_supports_parent_fallback_plural_rules_and_runtime_switching() {
        let mut localizer = ZsLocalizer::new(locale("en-GB"), locale("en"));
        localizer
            .add_ftl(
                locale("en"),
                r#"
save = Save
item-count = { $count ->
    [one] One item
   *[other] { $count } items
}
"#,
            )
            .expect("English resource should load");
        localizer
            .add_ftl(
                locale("zh-CN"),
                r#"
save = 保存
item-count = { $count } 个项目
"#,
            )
            .expect("Chinese resource should load");

        assert_eq!(localizer.text("save", "fallback"), "Save");
        let two_items = localizer.format(
            "item-count",
            &ZsMessageArgs::new().with("count", 2u32),
            "fallback",
        );
        assert!(two_items.contains('2'));
        assert!(two_items.contains("items"));

        localizer.set_locale(locale("zh-CN"));
        assert_eq!(localizer.text("save", "fallback"), "保存");
        assert!(localizer
            .format(
                "item-count",
                &ZsMessageArgs::new().with("count", 3u32),
                "fallback",
            )
            .contains("个项目"));
    }

    #[test]
    fn missing_message_uses_call_site_fallback() {
        let localizer = ZsLocalizer::new(locale("fr"), locale("en"));
        assert_eq!(
            localizer.text("missing", "Visible fallback"),
            "Visible fallback"
        );
    }

    #[test]
    fn malformed_resource_is_rejected() {
        let mut localizer = ZsLocalizer::new(locale("en"), locale("en"));
        assert!(localizer.add_ftl(locale("en"), "broken = {").is_err());
    }

    #[test]
    fn localizer_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ZsLocalizer>();
    }
}
