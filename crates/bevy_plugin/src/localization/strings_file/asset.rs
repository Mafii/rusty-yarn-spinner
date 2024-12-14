//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner-Unity/blob/462c735766a4c4881cd1ef1f15de28c83b2ba0a8/Runtime/StringTableEntry.cs>

use crate::prelude::*;
use anyhow::{anyhow, bail};
use bevy::asset::{io::Reader, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::utils::HashMap;
use sha2::{Digest, Sha256};
use std::fs;
use std::fs::File;
use std::path::Path;

pub(crate) fn strings_file_asset_plugin(app: &mut App) {
    app.init_asset::<StringsFile>()
        .init_asset_loader::<StringsFileAssetLoader>();
}

#[derive(Debug, Default)]
struct StringsFileAssetLoader;

impl AssetLoader for StringsFileAssetLoader {
    type Asset = StringsFile;
    type Settings = ();
    type Error = anyhow::Error;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let mut csv_reader = csv::Reader::from_reader(bytes.as_slice());
        let records: csv::Result<Vec<_>> = csv_reader.deserialize().collect();
        let strings_file = StringsFile::new_with_single_language(records?)?;
        Ok(strings_file)
    }

    fn extensions(&self) -> &[&str] {
        &["strings.csv"]
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Asset, TypePath)]
#[non_exhaustive]
pub(crate) struct StringsFile(HashMap<LineId, StringsFileRecord>);

impl StringsFile {
    pub(crate) fn new_with_single_language(records: Vec<StringsFileRecord>) -> Result<Self> {
        if let Some(language) = records.first().map(|record| &record.language) {
            for record in records.iter().skip(1) {
                if record.language != *language {
                    bail!("Loaded strings file with mixed languages records must have the same language. Expected \"{}\", got \"{}\" in record: {:#?}",
                    language,
                    record.language,
                    record)
                }
            }
        }
        let records = records
            .into_iter()
            .map(|record| (record.id.clone(), record))
            .collect::<HashMap<_, _>>();
        Ok(Self(records))
    }

    pub(crate) fn language(&self) -> Option<&Language> {
        self.0.iter().next().map(|(_id, record)| &record.language)
    }

    pub(crate) fn update_file(&mut self, mut other: Self) -> Result<bool> {
        let mut removed_lines = Vec::new();
        let Some(file) = other.0.iter().next().map(|(_, rec)| rec.file.clone()) else {
            return Ok(false);
        };
        if let Some(language) = self.language() {
            if language != other.language().unwrap() {
                bail!("Cannot update contents of strings file with another strings file that contains a different language. \
                Expected \"{:?}\", got \"{:?}\". This is a bug. Please report it at https://github.com/YarnSpinnerTool/YarnSpinner-Rust/issues/new",
                    self.language(), other.language())
            }
        }

        let single_yarn_file = other
            .0
            .values()
            .skip(1)
            .map(|rec| rec.file.as_str())
            .all(|other_file| other_file == file);

        let mut changed = false;
        for (id, record) in self.0.iter_mut() {
            if single_yarn_file && record.file != file {
                continue;
            }
            if let Some(other_record) = other.0.remove(id) {
                if records_equal_except_for_text(record, &other_record) {
                    continue;
                }
                let text_is_copied_from_base_language =
                    Lock::compute_from(&record.text) == record.lock;
                let text = if record.lock != other_record.lock
                    && !record.text.starts_with(UPDATE_PREFIX)
                    && !text_is_copied_from_base_language
                {
                    format!("{UPDATE_PREFIX}{}", &record.text)
                } else if !text_is_copied_from_base_language {
                    // not `other_record` because that one might not contain (NEEDS UPDATE)
                    record.text.clone()
                } else {
                    // This record's text was not translated, so we can safely overwrite it with the new text
                    other_record.text.clone()
                };
                let comment = combine_comments(&record.comment, &other_record.comment);

                changed = true;
                *record = StringsFileRecord {
                    text,
                    comment,
                    ..other_record
                };
            } else if single_yarn_file {
                removed_lines.push(id.clone());
                changed = true;
            }
        }
        for id in removed_lines {
            self.0.remove(&id);
        }
        if !other.0.is_empty() {
            changed = true;
            self.0.extend(other.0);
        }
        Ok(changed)
    }

    pub(crate) fn from_string_table(
        language: impl Into<Language>,
        string_table: impl IntoIterator<Item = (LineId, StringInfo)>,
    ) -> Result<Self> {
        let language = language.into();
        let mut records = HashMap::new();
        for (id, string_info) in string_table {
            if string_info.is_implicit_tag {
                bail!(
                    "Cannot build strings file from not fully tagged Yarn files (line {} in \"{}\" is not tagged).",
                    string_info.line_number,
                    string_info.file_name
                )
            }
            let lock = Lock::compute_from(&string_info.text);
            records.insert(
                id.clone(),
                StringsFileRecord {
                    language: language.clone(),
                    id,
                    text: string_info.text,
                    file: string_info.file_name,
                    node: string_info.node_name,
                    line_number: string_info.line_number,
                    lock,
                    comment: read_comments(string_info.metadata),
                },
            );
        }

        Ok(Self(records))
    }

    pub(crate) fn write_asset(&self, path: &Path) -> Result<()> {
        if let Some(parent_dir) = path.parent() {
            fs::create_dir_all(parent_dir).map_err(|e| {
                anyhow!(
                    "Failed to create dialogue asset subdirectory \"{}\": {e}",
                    parent_dir.display(),
                )
            })?;
        }
        let file = File::create(path)
            .map_err(|e| anyhow!("Failed to create strings file \"{}\": {e}", path.display(),))?;
        let mut writer = csv::Writer::from_writer(file);
        let mut records = self.0.iter().map(|(_, record)| record).collect::<Vec<_>>();
        records.sort_by(|lhs, rhs| {
            lhs.file
                .cmp(&rhs.file)
                .then(lhs.line_number.cmp(&rhs.line_number))
        });
        for record in records {
            writer.serialize(record)?;
        }
        writer.flush()?;
        Ok(())
    }

    pub(crate) fn get_offending_language(
        &self,
        expected_language: &Language,
    ) -> Option<&StringsFileRecord> {
        self.0
            .values()
            .find(|record| &record.language != expected_language)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&LineId, &StringsFileRecord)> {
        self.0.iter()
    }

    pub(crate) fn records(&self) -> impl Iterator<Item = &StringsFileRecord> {
        self.0.values()
    }
}

fn records_equal_except_for_text(lhs: &StringsFileRecord, rhs: &StringsFileRecord) -> bool {
    lhs.language == rhs.language
        && lhs.id == rhs.id
        && lhs.file == rhs.file
        && lhs.node == rhs.node
        && lhs.line_number == rhs.line_number
        && lhs.lock == rhs.lock
        && lhs.comment == rhs.comment
}
const UPDATE_PREFIX: &str = "(NEEDS UPDATE) ";

fn combine_comments(full_old_comment: &str, new_metadata: &str) -> String {
    let translator_comment = extract_translator_comment(full_old_comment);
    let new_metadata = (!new_metadata.is_empty()).then_some(new_metadata);
    [translator_comment, new_metadata]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(LINE_METADATA_PREFIX_SEPARATOR)
}

fn extract_translator_comment(comment: &str) -> Option<&str> {
    let mut split = comment.split(LINE_METADATA_PREFIX);
    split
        .next()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches(LINE_METADATA_PREFIX_SEPARATOR))
}

const LINE_METADATA_PREFIX: &str = "Line metadata: ";
const LINE_METADATA_PREFIX_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct StringsFileRecord {
    /// The language that the line is written in.
    pub(crate) language: Language,
    /// The line ID for this line. This value will be the same across all localizations.
    pub(crate) id: LineId,
    /// The text of this line, in the language specified by [`language`](StringsFileRecord::language).
    pub(crate) text: String,
    /// The name of the Yarn script in which this line was originally found.
    pub(crate) file: String,
    /// The name of the node in which this line was originally found.
    ///
    /// This node can be found in the file indicated by [`file`](StringsFileRecord::file).
    pub(crate) node: String,

    /// The 1-indexed line number in the file indicated by [`file`](StringsFileRecord::file) at
    /// which the original version of this line can be found.
    pub(crate) line_number: usize,
    /// A string used as part of a mechanism for checking if translated
    /// versions of this string are out of date.
    ///
    /// This field contains the first 8 characters of the SHA-256 hash of
    /// the line's text as it appeared in the base localization CSV file.
    ///
    /// When a new StringTableEntry is created in a localized CSV file for a
    /// .Yarn file, the Lock value is copied over from the base CSV file,
    /// and used for the translated entry.
    ///
    /// Because the base localization CSV is regenerated every time the
    /// .Yarn file is imported, the base localization Lock value will change
    /// if a line's text changes. This means that if the base lock and
    /// translated lock differ, the translated line is out of date, and
    /// needs to be updated.
    pub(crate) lock: Lock,
    /// A comment used to describe this line to translators.
    pub(crate) comment: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Debug, PartialEq, Hash, Serialize, Deserialize)]
pub(crate) struct Lock(String);

impl Lock {
    /// Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner-Unity/blob/462c735766a4c4881cd1ef1f15de28c83b2ba0a8/Editor/Importers/YarnImporter.cs#L149>
    pub(crate) fn compute_from(text: &str) -> Self {
        const MAX_CHARS: usize = 8;
        let hash = Sha256::digest(text);
        let hex = format!("{hash:x}");
        let lock = hex.chars().take(MAX_CHARS).collect();
        Self(lock)
    }
}

/// Generates a string with the line metadata. This string is intended
/// to be used in the "comment" column of a strings table CSV. Because
/// of this, it will ignore the line ID if it exists (which is also
/// part of the line metadata).
///
/// ## Return value
/// A string prefixed with "Line metadata: ", followed by each
/// piece of metadata separated by whitespace. If no metadata exists or
/// only the line ID is part of the metadata, returns an empty string
/// instead.
fn read_comments(metadata: impl IntoIterator<Item = String>) -> String {
    // Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner-Unity/blob/462c735766a4c4881cd1ef1f15de28c83b2ba0a8/Editor/Importers/YarnProjectImporter.cs#L652>
    let cleaned_metadata: Vec<_> = metadata
        .into_iter()
        .filter(|metadata| !metadata.starts_with("line:"))
        .collect();
    if cleaned_metadata.is_empty() {
        String::new()
    } else {
        format!("{LINE_METADATA_PREFIX}{}", cleaned_metadata.join(" "))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn combines_comments_without_change() {
        let old = "Foo, Line metadata: Bar";
        let new = "Line metadata: Bar";
        let combined = combine_comments(old, new);
        assert_eq!(old, &combined)
    }

    #[test]
    fn combines_comments_with_deletion() {
        let old = "Foo, Line metadata: Bar";
        let new = "";
        let combined = combine_comments(old, new);
        assert_eq!("Foo", &combined)
    }

    #[test]
    fn combines_comments_with_insertion() {
        let old = "Foo, Line metadata: Bar";
        let new = "Line metadata: Bar, Baz";
        let combined = combine_comments(old, new);
        assert_eq!("Foo, Line metadata: Bar, Baz", &combined)
    }

    #[test]
    fn combines_comments_with_change() {
        let old = "Foo, Line metadata: Bar";
        let new = "Line metadata: Baz";
        let combined = combine_comments(old, new);
        assert_eq!("Foo, Line metadata: Baz", &combined)
    }

    #[test]
    fn combines_comments_without_meta() {
        let old = "Foo";
        let new = "";
        let combined = combine_comments(old, new);
        assert_eq!(old, &combined)
    }

    #[test]
    fn combines_comments_with_new_meta() {
        let old = "Foo";
        let new = "Line metadata: Bar";
        let combined = combine_comments(old, new);
        assert_eq!("Foo, Line metadata: Bar", &combined)
    }

    #[test]
    fn combines_comments_with_only_same_meta() {
        let old = "Line metadata: Bar";
        let new = "Line metadata: Bar";
        let combined = combine_comments(old, new);
        assert_eq!(old, &combined)
    }

    #[test]
    fn combines_empty_comments() {
        let old = "";
        let new = "";
        let combined = combine_comments(old, new);
        assert_eq!(old, &combined)
    }

    #[test]
    fn combines_comments_with_only_new_meta() {
        let old = "";
        let new = "Line metadata: Bar";
        let combined = combine_comments(old, new);
        assert_eq!(new, &combined)
    }

    #[test]
    fn combines_comments_with_only_changed_meta() {
        let old = "Line metadata: Bar";
        let new = "Line metadata: Baz";
        let combined = combine_comments(old, new);
        assert_eq!(new, &combined)
    }
}
