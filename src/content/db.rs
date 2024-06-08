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
    pub value: Option<String>,
}

impl SlideContent {
    pub const UNION_SELECT: &'static str = "SELECT \"id\", 'display_output' AS \"for_type\", \"display_output_id\" AS \"for_id\", \"key\", \"value\" FROM \"display_output_content\" \
        UNION ALL SELECT \"id\", 'slide_type' AS \"for_type\", \"slide_type_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_type_content\" \
        UNION ALL SELECT \"id\", 'slide_group' AS \"for_type\", \"slide_group_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_group_content\" \
        UNION ALL SELECT \"id\", 'slide' AS \"for_type\", \"slide_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_content\" \
        UNION ALL SELECT \"id\", 'slide_deck' AS \"for_type\", \"slide_deck_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_deck_content\" \
        UNION ALL SELECT \"id\", 'slide_deck_section' AS \"for_type\", \"slide_deck_section_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_deck_section_content\" \
        UNION ALL SELECT \"id\", 'slide_deck_slide' AS \"for_type\", \"slide_deck_slide_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_deck_slide_content\"";

    // Uses parameters `:slide_deck_slide_id` and `:display_output_id`
    pub const CONTENT_FOR_SLIDE_DECK_SLIDE: &'static str = "\
        WITH \"cte_related_ids\" AS ( \
            SELECT \
                \"slide_deck_slides\".\"id\" AS \"slide_deck_slide_id\", \
                \"slide_deck_sections\".\"id\" AS \"slide_deck_section_id\", \
                \"slide_decks\".\"id\" AS \"slide_deck_id\", \
                \"slides\".\"id\" AS \"slide_id\", \
                \"slide_groups\".\"id\" AS \"slide_group_id\", \
                \"slide_types\".\"id\" AS \"slide_type_id\" \
            FROM \"slide_deck_slides\" \
            LEFT JOIN \"slide_deck_sections\" \
                ON \"slide_deck_slides\".\"slide_deck_section_id\" = \"slide_deck_sections\".\"id\" \
            LEFT JOIN \"slide_decks\" \
                ON \"slide_deck_sections\".\"slide_deck_id\" = \"slide_decks\".\"id\" \
            LEFT JOIN \"slides\" \
                ON \"slide_deck_slides\".\"slide_id\" = \"slides\".\"id\" \
            LEFT JOIN \"slide_groups\" \
                ON COALESCE(\"slide_deck_sections\".\"slide_group_id\", \"slides\".\"slide_group_id\") = \"slide_groups\".\"id\" \
            LEFT JOIN \"slide_types\" \
                ON COALESCE(\"slide_deck_slides\".\"slide_type_override_id\", \"slide_deck_sections\".\"slide_type_override_id\", \"slides\".\"slide_type_id\") = \"slide_types\".\"id\" \
            WHERE \
                \"slide_deck_slides\".\"id\" = :slide_deck_slide_id \
        ), \"cte_all_values\" AS ( \
            SELECT \
                \"key\", \"value\", 1 AS \"priority\" \
            FROM \"slide_deck_slide_content\" \
            WHERE \
                \"slide_deck_slide_id\" IN (SELECT \"slide_deck_slide_id\" FROM \"cte_related_ids\") \
            UNION ALL \
            SELECT \
                \"key\", \"value\", 2 AS \"priority\" \
            FROM \"slide_deck_section_content\" \
            WHERE \
                \"slide_deck_section_id\" IN (SELECT \"slide_deck_section_id\" FROM \"cte_related_ids\") \
            UNION ALL \
            SELECT \
                \"key\", \"value\", 3 AS \"priority\" \
            FROM \"slide_deck_content\" \
            WHERE \
                \"slide_deck_id\" IN (SELECT \"slide_deck_id\" FROM \"cte_related_ids\") \
            UNION ALL \
            SELECT \
                \"key\", \"value\", 4 AS \"priority\" \
            FROM \"slide_content\" \
            WHERE \
                \"slide_id\" IN (SELECT \"slide_id\" FROM \"cte_related_ids\") \
            UNION ALL \
            SELECT \
                \"key\", \"value\", 5 AS \"priority\" \
            FROM \"slide_group_content\" \
            WHERE \
                \"slide_group_id\" IN (SELECT \"slide_group_id\" FROM \"cte_related_ids\") \
            UNION ALL \
            SELECT \
                \"key\", \"value\", 6 AS \"priority\" \
            FROM \"slide_type_content\" \
            WHERE \
                \"slide_type_id\" IN (SELECT \"slide_type_id\" FROM \"cte_related_ids\") \
            UNION ALL \
            SELECT \
                \"key\", \"value\", 7 AS \"priority\" \
            FROM \"display_output_content\" \
            WHERE \
                \"display_output_id\" = :display_output_id \
        ), \"cte_sorted_values\" AS ( \
            SELECT \
                \"key\", \
                \"value\", \
                row_number() OVER (PARTITION BY \"key\" ORDER BY \"priority\") AS \"priority\" \
            FROM \"cte_all_values\" \
        ), \"cte_top_values\" AS ( \
            SELECT \"key\", \"value\" FROM \"cte_sorted_values\" WHERE \"priority\" = 1 \
        ) SELECT * FROM \"cte_top_values\"";

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
            value: row
                .get("value")
                .expect("Failed to get value from database row"),
        }
    }

    pub fn select_stmt(for_type: ContentFor) -> &'static str {
        match for_type {
            ContentFor::DisplayOutput => "SELECT \"id\", 'display_output' AS \"for_type\", \"display_output_id\" AS \"for_id\", \"key\", \"value\" FROM \"display_output_content\"",
            ContentFor::SlideType => "SELECT \"id\", 'slide_type' AS \"for_type\", \"slide_type_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_type_content\"",
            ContentFor::SlideGroup => "SELECT \"id\", 'slide_group' AS \"for_type\", \"slide_group_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_group_content\"",
            ContentFor::Slide => "SELECT \"id\", 'slide' AS \"for_type\", \"slide_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_content\"",
            ContentFor::SlideDeck => "SELECT \"id\", 'slide_deck' AS \"for_type\", \"slide_deck_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_deck_content\"",
            ContentFor::SlideDeckSection => "SELECT \"id\", 'slide_deck_section' AS \"for_type\", \"slide_deck_section_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_deck_section_content\"",
            ContentFor::SlideDeckSlide => "SELECT \"id\", 'slide_deck_slide' AS \"for_type\", \"slide_deck_slide_id\" AS \"for_id\", \"key\", \"value\" FROM \"slide_deck_slide_content\"",
        }
    }

    pub fn insert_stmt(for_type: ContentFor) -> &'static str {
        match for_type {
            ContentFor::DisplayOutput => "INSERT INTO \"display_output_content\" (\"id\", \"display_output_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
            ContentFor::SlideType => "INSERT INTO \"slide_type_content\" (\"id\", \"slide_type_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
            ContentFor::SlideGroup => "INSERT INTO \"slide_group_content\" (\"id\", \"slide_group_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
            ContentFor::Slide => "INSERT INTO \"slide_content\" (\"id\", \"slide_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
            ContentFor::SlideDeck => "INSERT INTO \"slide_deck_content\" (\"id\", \"slide_deck_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
            ContentFor::SlideDeckSection => "INSERT INTO \"slide_deck_section_content\" (\"id\", \"slide_deck_section_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
            ContentFor::SlideDeckSlide => "INSERT INTO \"slide_deck_slide_content\" (\"id\", \"slide_deck_slide_id\", \"key\", \"value\") VALUES (:id, :for_id, :key, :value)",
        }
    }

    pub fn update_stmt(for_type: ContentFor) -> &'static str {
        match for_type {
            ContentFor::DisplayOutput => "UPDATE \"display_output_content\" SET \"display_output_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
            ContentFor::SlideType => "UPDATE \"slide_type_content\" SET \"slide_type_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
            ContentFor::SlideGroup => "UPDATE \"slide_group_content\" SET \"slide_group_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
            ContentFor::Slide => "UPDATE \"slide_content\" SET \"slide_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
            ContentFor::SlideDeck => "UPDATE \"slide_deck_content\" SET \"slide_deck_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
            ContentFor::SlideDeckSection => "UPDATE \"slide_deck_section_content\" SET \"slide_deck_section_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
            ContentFor::SlideDeckSlide => "UPDATE \"slide_deck_slide_content\" SET \"slide_deck_slide_id\" = :for_id, \"key\" = :key, \"value\" = :value WHERE \"id\" = :id",
        }
    }
}
