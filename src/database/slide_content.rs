use rusqlite::{
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
    Row, ToSql,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub enum ContentFor {
    DisplayOutput,
    SlideType,
    SlideGroup,
    Slide,
    SlideDeck,
    SlideDeckSection,
    SlideDeckSlide,
}
impl ToSql for ContentFor {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(match self {
            Self::DisplayOutput => "display_output",
            Self::SlideType => "slide_type",
            Self::SlideGroup => "slide_group",
            Self::Slide => "slide",
            Self::SlideDeck => "slide_deck",
            Self::SlideDeckSection => "slide_deck_section",
            Self::SlideDeckSlide => "slide_deck_slide",
        }
        .into())
    }
}
impl FromSql for ContentFor {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_str() {
            Ok("display_output") => Ok(Self::DisplayOutput),
            Ok("slide_type") => Ok(Self::SlideType),
            Ok("slide_group") => Ok(Self::SlideGroup),
            Ok("slide") => Ok(Self::Slide),
            Ok("slide_deck") => Ok(Self::SlideDeck),
            Ok("slide_deck_section") => Ok(Self::SlideDeckSection),
            Ok("slide_deck_slide") => Ok(Self::SlideDeckSlide),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SlideContent {
    pub id: Option<Uuid>,
    pub for_type: ContentFor,
    pub for_id: Uuid,
    pub key: String,
    pub content: Option<String>,
}

impl SlideContent {
    pub const UNION_SELECT: &'static str = "SELECT \"id\", 'display_output' AS \"for_type\", \"display_output_id\" AS \"for_id\", \"key\", \"content\" FROM \"display_output_content\" \
        UNION ALL SELECT \"id\", 'slide_type' AS \"for_type\", \"slide_type_id\" AS \"for_id\", \"key\", \"content\" FROM \"slide_type_content\" \
        UNION ALL SELECT \"id\", 'slide_group' AS \"for_type\", \"slide_group_id\" AS \"for_id\", \"key\", \"content\" FROM \"slide_group_content\" \
        UNION ALL SELECT \"id\", 'slide' AS \"for_type\", \"slide_id\" AS \"for_id\", \"key\", \"content\" FROM \"slide_content\" \
        UNION ALL SELECT \"id\", 'slide_deck' AS \"for_type\", \"slide_deck_id\" AS \"for_id\", \"key\", \"content\" FROM \"slide_deck_content_overrides\" \
        UNION ALL SELECT \"id\", 'slide_deck_section' AS \"for_type\", \"slide_deck_section_id\" AS \"for_id\", \"key\", \"content\" FROM \"slide_deck_section_content_overrides\" \
        UNION ALL SELECT \"id\", 'slide_deck_slide' AS \"for_type\", \"slide_deck_slide_id\" AS \"for_id\", \"key\", \"content\" FROM \"slide_deck_slide_content_overrides\"";

    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row
                .get("id")
                .expect("Failed to get value from database row"),
            for_type: row
                .get("for_type")
                .expect("Failed to get value from database row"),
            for_id: row
                .get("for_id")
                .expect("Failed to get value from database row"),
            key: row
                .get("key")
                .expect("Failed to get value from database row"),
            content: row
                .get("content")
                .expect("Failed to get value from database row"),
        }
    }

    pub fn insert_stmt(for_type: ContentFor) -> &'static str {
        match for_type {
            ContentFor::DisplayOutput => "INSERT INTO \"display_output_content\" (\"id\", \"display_output_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
            ContentFor::SlideType => "INSERT INTO \"slide_type_content\" (\"id\", \"slide_type_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
            ContentFor::SlideGroup => "INSERT INTO \"slide_group_content\" (\"id\", \"slide_group_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
            ContentFor::Slide => "INSERT INTO \"slide_content\" (\"id\", \"slide_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
            ContentFor::SlideDeck => "INSERT INTO \"slide_deck_content_overrides\" (\"id\", \"slide_deck_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
            ContentFor::SlideDeckSection => "INSERT INTO \"slide_deck_section_content_overrides\" (\"id\", \"slide_deck_section_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
            ContentFor::SlideDeckSlide => "INSERT INTO \"slide_deck_slide_content_overrides\" (\"id\", \"slide_deck_slide_id\", \"key\", \"content\") VALUES (:id, :for_id, :key, :content);",
        }
    }

    pub fn update_stmt(for_type: ContentFor) -> &'static str {
        match for_type {
            ContentFor::DisplayOutput => "UPDATE \"display_output_content\" SET \"display_output_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
            ContentFor::SlideType => "UPDATE \"slide_type_content\" SET \"slide_type_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
            ContentFor::SlideGroup => "UPDATE \"slide_group_content\" SET \"slide_group_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
            ContentFor::Slide => "UPDATE \"slide_content\" SET \"slide_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
            ContentFor::SlideDeck => "UPDATE \"slide_deck_content_overrides\" SET \"slide_deck_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
            ContentFor::SlideDeckSection => "UPDATE \"slide_deck_section_content_overrides\" SET \"slide_deck_section_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
            ContentFor::SlideDeckSlide => "UPDATE \"slide_deck_slide_content_overrides\" SET \"slide_deck_slide_id\" = :for_id, \"key\" = :key, \"content\" = :content WHERE \"id\" = :id;",
        }
    }
}
