use crate::plugin::AssetRoot;
use crate::{localization::line_id_generation::LineIdUpdateSystemSet, prelude::*};
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};

pub(crate) fn strings_file_updating_plugin(app: &mut App) {
    app.add_event::<UpdateAllStringsFilesForStringTableEvent>()
        .add_systems(
            Update,
            (update_all_strings_files_for_string_table
                .pipe(panic_on_err)
                .after(LineIdUpdateSystemSet)
                .in_set(YarnSpinnerSystemSet)
                .run_if(
                    in_development
                        .and(has_localizations)
                        .and(resource_exists::<YarnProject>)
                        .and(events_in_queue::<UpdateAllStringsFilesForStringTableEvent>()),
                ),)
                .chain(),
        );
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Reflect, Event)]
#[reflect(Debug, Default, PartialEq)]
pub(crate) struct UpdateAllStringsFilesForStringTableEvent(
    pub(crate) std::collections::HashMap<LineId, StringInfo>,
);

fn update_all_strings_files_for_string_table(
    mut events: ResMut<Events<UpdateAllStringsFilesForStringTableEvent>>,
    mut strings_files: ResMut<Assets<StringsFile>>,
    asset_server: Res<AssetServer>,
    project: Res<YarnProject>,
    mut languages_to_handles: Local<HashMap<Language, Handle<StringsFile>>>,
    mut expected_file_names: Local<HashSet<String>>,
    asset_root: Res<AssetRoot>,
) -> SystemResult {
    let localizations = project.localizations.as_ref().unwrap();
    if localizations.translations.is_empty() {
        events.clear();
        return Ok(());
    }

    for localization in &localizations.translations {
        let language = &localization.language;
        let path = localization.strings_file.as_path();
        let asset_path = path.to_string_lossy().replace('\\', "/");
        let handle = asset_server.load(asset_path);
        languages_to_handles.insert(language.clone(), handle);
    }
    if languages_to_handles.is_empty() {
        events.clear();
        return Ok(());
    }
    if languages_to_handles
        .values()
        .any(|h| !strings_files.contains(h))
    {
        return Ok(());
    }
    if expected_file_names.is_empty() {
        expected_file_names.extend(
            project
                .compilation
                .string_table
                .values()
                .map(|string_info| string_info.file_name.clone()),
        );
    }

    let mut dirty_paths = HashSet::new();
    for string_table in events.drain().map(|e| e.0) {
        let file_names: HashSet<_> = string_table
            .values()
            .map(|s| s.file_name.as_str())
            .collect();
        let file_names = file_names.into_iter().collect::<Vec<_>>().join(", ");
        for (language, strings_file_handle) in languages_to_handles.clone() {
            let strings_file = strings_files.get_mut(&strings_file_handle).unwrap();
            lint_strings_file(
                strings_file,
                &expected_file_names,
                &asset_server,
                &strings_file_handle,
            );

            let strings_file_path = localizations.strings_file_path(language.clone()).unwrap();

            let new_strings_file = match StringsFile::from_string_table(
                language.clone(),
                string_table.clone(),
            ) {
                Ok(new_strings_file) => new_strings_file,
                Err(e) => {
                    if project.development_file_generation == DevelopmentFileGeneration::Full {
                        debug!("Updating \"{}\" soon (lang: {language}) because the following Yarn files were changed or loaded but do not have full line IDs yet: {file_names}",
                            strings_file_path.display())
                    } else {
                        error!(
                            "Tried to update \"{}\" (lang: {language}) because the following Yarn files were changed or loaded: {file_names}, but couldn't because: {e}",
                            strings_file_path.display(),
                        );
                    }
                    continue;
                }
            };
            if strings_file.update_file(new_strings_file)? {
                dirty_paths.insert((strings_file_handle, strings_file_path));

                info!(
                    "Updated \"{}\" (lang: {language}) because the following Yarn files were changed or loaded: {file_names}",
                    strings_file_path.display(),
                );
            }
        }
    }
    languages_to_handles.clear();
    for (handle, path) in &dirty_paths {
        let strings_file = strings_files.get(handle).unwrap();
        let path = asset_root.0.join(path);
        strings_file.write_asset(&path)?;
    }
    Ok(())
}

fn lint_strings_file(
    strings_file: &StringsFile,
    expected_file_names: &HashSet<String>,
    asset_server: &AssetServer,
    handle: &Handle<StringsFile>,
) {
    let actual_file_names: HashSet<_> =
        strings_file.records().map(|rec| rec.file.clone()).collect();
    let superfluous_file_names = actual_file_names
        .difference(expected_file_names)
        .map(|name| name.to_owned())
        .collect::<Vec<_>>()
        .join(", ");
    if !superfluous_file_names.is_empty() {
        let source = asset_server
            .get_path(handle)
            .map(|asset_path| format!("at {}", asset_path.path().display()))
            .unwrap_or_else(|| "created at runtime".to_owned());
        warn!(
            "Strings file {source} contains the following strings for Yarn files were not found in the project: {superfluous_file_names}. \
            Either you forgot to add these files to the project or the strings belonged to files that were deleted. \
            You may want to delete these entries from the strings file manually. Yarn Spinner will not do this for you because it may lead to loss of work.",
        );
    }
}
