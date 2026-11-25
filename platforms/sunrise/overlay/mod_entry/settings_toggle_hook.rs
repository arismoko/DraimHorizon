fn claim_borders_settings_toggle(
    game: &mut crate::game::Game<'_>,
    owner: &'static str,
    spec: &'static SettingSpec,
    value: SettingValue,
) -> ::core::ops::ControlFlow<Result<bool, SettingAccessError>> {
    if owner != crate::game::claim_borders::ID
        || spec.key.as_str() != "open_menu"
        || !matches!(value, SettingValue::Toggle(true))
    {
        return ::core::ops::ControlFlow::Continue(());
    }
    // The toggle is a button: leave it off (nothing is stored), then do what the Horizon settings
    // button does.
    game.sync_native_setting_ui_row(owner, spec, SettingValue::Toggle(false));
    crate::game::claim_borders_client::open_menu_from_options(game);
    crate::game::claim_borders_menu_view::open_settings();
    ::core::ops::ControlFlow::Break(Ok(false))
}
