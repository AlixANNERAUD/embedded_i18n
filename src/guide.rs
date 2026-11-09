//! User guide for `embedded-i18n`, organized in four parts:
//!
//! - [tutorials]: learn by building a translated program, step by step;
//! - [how_to]: recipes for specific tasks;
//! - [reference](mod@reference): the macros, options, file formats and errors, exhaustively;
//! - [explanation]: how the crate works and why it is designed this way.

/// Tutorials: learn by building a translated program, step by step.
pub mod tutorials {
    /// From an empty project to a program translated into two languages, without allocator.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/tutorials/getting_started.md")]
    pub mod getting_started {}
}

/// How-to guides: recipes for specific tasks.
pub mod how_to {
    /// Embed several locales and select one at runtime.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/switch_language_at_runtime.md")]
    pub mod switch_language_at_runtime {}

    /// Translate each call into a given locale, without a global current locale.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/localize_per_request.md")]
    pub mod localize_per_request {}

    /// Build the same code once per language, locally and in CI.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/build_one_firmware_per_language.md")]
    pub mod build_one_firmware_per_language {}

    /// Read GNU gettext PO and MO, YAML, TOML or Fluent files.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/use_gettext_po_files.md")]
    pub mod use_gettext_po_files {}

    /// Fail the build on untranslated strings with strict mode.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/ship_complete_translations.md")]
    pub mod ship_complete_translations {}

    /// Test layouts and fonts with the `pseudo` locale before translating.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/pseudo_localize.md")]
    pub mod pseudo_localize {}

    /// Mirror layouts and isolate placeholders for right-to-left languages.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/support_right_to_left_languages.md")]
    pub mod support_right_to_left_languages {}

    /// Format localized dates and times without `libc` or allocation.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/format_dates_and_times.md")]
    pub mod format_dates_and_times {}

    /// Get the Unicode ranges of a language to subset fonts.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/subset_fonts.md")]
    pub mod subset_fonts {}

    /// Move a crate from rust-i18n, item by item.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/migrate_from_rust_i18n.md")]
    pub mod migrate_from_rust_i18n {}

    /// Upgrade a crate from `embedded-i18n` 0.1, item by item.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/how_to/migrate_from_0_1.md")]
    pub mod migrate_from_0_1 {}
}

/// Reference: the macros, options, file formats and errors, exhaustively.
pub mod reference {
    /// The `t!` macro: grammar, options, result types, placeholders and compile errors.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/t_macro.md")]
    pub mod t_macro {}

    /// The `i18n!` arguments, `[package.metadata.i18n]` keys, environment overrides and errors.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/configuration.md")]
    pub mod configuration {}

    /// Locale file formats, key layout, `backend-*` features and errors.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/locale_files.md")]
    pub mod locale_files {}

    /// CLDR plural categories, built-in rules and gettext `Plural-Forms`.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/plural_rules.md")]
    pub mod plural_rules {}

    /// `set_locale`, `Locale`, `locale!`, `available_locales!`, `system_locale` and `is_right_to_left`.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/runtime_api.md")]
    pub mod runtime_api {}

    /// `DateTimeFormatter`, its tokens and the supported languages.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/date_time.md")]
    pub mod date_time {}

    /// The Unicode ranges of each language and the range helpers.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/unicode_ranges.md")]
    pub mod unicode_ranges {}

    /// Cargo features, what needs an allocator, and the supported targets.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/reference/features_and_no_std.md")]
    pub mod features_and_no_std {}
}

/// Explanation: how the crate works and why it is designed this way.
pub mod explanation {
    /// What the macros generate, why `i18n!` comes first, and the ROM and RAM cost.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/explanation/how_it_works.md")]
    pub mod how_it_works {}

    /// The packed locale identifier, per-crate resolution and their limits.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/explanation/runtime_locale_model.md")]
    pub mod runtime_locale_model {}

    /// Why compile errors, `Display` values and one embedded locale by default.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/explanation/design_choices.md")]
    pub mod design_choices {}

    /// How `embedded-i18n` compares with rust-i18n, Fluent and gettext.
    ///
    #[doc = include_str!("../docs/rustdoc-links.md")]
    #[doc = include_str!("../docs/explanation/comparison.md")]
    pub mod comparison {}
}
