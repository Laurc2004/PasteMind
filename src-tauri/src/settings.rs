use crate::{
  error::{AppError, AppResult},
  hotkey,
  models::{Settings, SettingsPatch},
  storage::Storage
};

pub fn load_or_default(storage: &Storage) -> AppResult<Settings> {
  if let Some(settings) = storage.load_settings()? {
    validate_settings(&settings)?;
    Ok(settings)
  } else {
    let settings = Settings::default();
    storage.save_settings(&settings)?;
    Ok(settings)
  }
}

pub fn merge_and_validate(current: &Settings, patch: SettingsPatch) -> AppResult<Settings> {
  let mut next = current.clone();
  next.apply_patch(patch);
  next.hotkey = hotkey::normalize_hotkey(&next.hotkey);
  validate_settings(&next)?;
  Ok(next)
}

pub fn validate_settings(settings: &Settings) -> AppResult<()> {
  if settings.max_items == 0 || settings.max_items > 5_000 {
    return Err(AppError::InvalidSettings(
      "max_items must be in [1, 5000]".to_string()
    ));
  }

  if settings.max_image_mb == 0 || settings.max_image_mb > 100 {
    return Err(AppError::InvalidSettings(
      "max_image_mb must be in [1, 100]".to_string()
    ));
  }

  if settings.ui_locale != "zh-CN" && settings.ui_locale != "en-US" {
    return Err(AppError::InvalidSettings(
      "ui_locale must be one of: zh-CN, en-US".to_string()
    ));
  }

  hotkey::validate_hotkey(&settings.hotkey)?;

  Ok(())
}
