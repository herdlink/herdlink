use crate::{
    AppState,
    auth::AuthUser,
    communities::{self, OpenCommunity},
    error::{AppError, Result},
    validation,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuestionKind {
    ShortText,
    SingleChoice,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    id: String,
    prompt: String,
    kind: QuestionKind,
    #[serde(default)]
    options: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    key: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateSurvey {
    title: String,
    #[serde(default)]
    description: String,
    questions: Vec<Question>,
    communities: Vec<Target>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Audience {
    communities: Vec<Target>,
}
#[derive(Serialize, FromRow)]
pub struct TargetView {
    id: Uuid,
    slug: String,
    name: String,
}
#[derive(FromRow)]
struct Row {
    id: Uuid,
    creator_id: Uuid,
    title: String,
    description: String,
    questions_json: String,
    created_at: String,
}
#[derive(Serialize)]
pub struct QuestionResults {
    counts: std::collections::BTreeMap<String, i64>,
    texts: Vec<String>,
}
#[derive(Serialize)]
pub struct Survey {
    id: Uuid,
    creator_id: Uuid,
    title: String,
    description: String,
    created_at: String,
    questions: Vec<Question>,
    communities: Vec<TargetView>,
    audience_count: i64,
    response_count: i64,
    can_respond: bool,
    answers: Option<Vec<String>>,
    results: Option<Vec<QuestionResults>>,
}
fn targets(communities: Vec<Target>) -> Result<Vec<Target>> {
    if communities.is_empty() || communities.len() > 30 {
        return Err(AppError::bad_request("select 1-30 disease communities"));
    }
    communities
        .into_iter()
        .map(|t| {
            Ok(Target {
                key: validation::slug(&t.key)?,
                name: validation::text(&t.name, 100)?,
            })
        })
        .collect()
}
fn questions(mut questions: Vec<Question>) -> Result<Vec<Question>> {
    if questions.is_empty() || questions.len() > 10 {
        return Err(AppError::bad_request("include 1-10 questions"));
    }
    let mut ids = std::collections::BTreeSet::new();
    for q in &mut questions {
        q.id = validation::text(&q.id, 80)?;
        if !ids.insert(q.id.clone()) {
            return Err(AppError::bad_request("question IDs must be unique"));
        }
        q.prompt = validation::text(&q.prompt, 500)?;
        match q.kind {
            QuestionKind::ShortText => q.options.clear(),
            QuestionKind::SingleChoice => {
                if !(2..=8).contains(&q.options.len()) {
                    return Err(AppError::bad_request("choice questions need 2-8 options"));
                }
                q.options = q
                    .options
                    .iter()
                    .map(|o| validation::text(o, 150))
                    .collect::<Result<_>>()?;
                if q.options
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != q.options.len()
                {
                    return Err(AppError::bad_request("options must be unique"));
                }
            }
        }
    }
    Ok(questions)
}
pub async fn audience(
    State(state): State<AppState>,
    _auth: AuthUser,
    Json(input): Json<Audience>,
) -> Result<Json<serde_json::Value>> {
    let targets = targets(input.communities)?;
    let keys = serde_json::to_string(&targets.iter().map(|t| &t.key).collect::<Vec<_>>())
        .map_err(AppError::internal)?;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT m.user_id) FROM community_members m JOIN communities c ON c.id=m.community_id WHERE c.slug IN (SELECT value FROM json_each(?1)) OR lower(hex(c.id)) IN (SELECT replace(value,'-','') FROM json_each(?1))")
        .bind(keys).fetch_one(&state.db).await?;
    Ok(Json(serde_json::json!({"audience_count":count})))
}
pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(input): Json<CreateSurvey>,
) -> Result<(StatusCode, Json<Survey>)> {
    let title = validation::text(&input.title, 200)?;
    let description = input.description.trim();
    if description.chars().count() > 3000 || description.contains('\0') {
        return Err(AppError::bad_request("invalid survey description"));
    }
    let questions = questions(input.questions)?;
    let targets = targets(input.communities)?;
    let mut communities = std::collections::BTreeSet::new();
    for target in targets {
        let Json(community) = communities::open(
            State(state.clone()),
            auth.clone(),
            Path(target.key),
            Some(Json(OpenCommunity {
                name: Some(target.name),
                preview: true,
            })),
        )
        .await?;
        communities.insert(community.id);
    }
    let id = Uuid::new_v4();
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO surveys (id,creator_id,title,description,questions_json) VALUES (?1,?2,?3,?4,?5)")
        .bind(id).bind(auth.id).bind(title).bind(description).bind(serde_json::to_string(&questions).map_err(AppError::internal)?).execute(&mut *tx).await?;
    for community in communities {
        sqlx::query("INSERT INTO survey_communities (survey_id,community_id) VALUES (?1,?2)")
            .bind(id)
            .bind(community)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(read(&state, id, auth.id, true).await?),
    ))
}
#[derive(Default, Deserialize)]
pub struct ListQuery {
    limit: Option<i64>,
    offset: Option<i64>,
    community: Option<Uuid>,
}
pub async fn list(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(input): Query<ListQuery>,
) -> Result<Json<Vec<Survey>>> {
    let (limit, offset) = validation::Pagination {
        limit: input.limit,
        offset: input.offset,
    }
    .bounds()?;
    let ids: Vec<Uuid> = sqlx::query_scalar("SELECT s.id FROM surveys s WHERE (s.creator_id=?1 OR EXISTS (SELECT 1 FROM survey_communities sc JOIN community_members m ON m.community_id=sc.community_id WHERE sc.survey_id=s.id AND m.user_id=?1)) AND (?2 IS NULL OR EXISTS (SELECT 1 FROM survey_communities sc WHERE sc.survey_id=s.id AND sc.community_id=?2)) ORDER BY s.created_at DESC,s.id LIMIT ?3 OFFSET ?4")
        .bind(auth.id).bind(input.community).bind(limit).bind(offset).fetch_all(&state.db).await?;
    let mut surveys = Vec::new();
    for id in ids {
        surveys.push(read(&state, id, auth.id, false).await?);
    }
    Ok(Json(surveys))
}
async fn read(state: &AppState, id: Uuid, user: Uuid, include_results: bool) -> Result<Survey> {
    let row: Row = sqlx::query_as("SELECT * FROM surveys WHERE id=?1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    let can_respond: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM survey_communities sc JOIN community_members m ON m.community_id=sc.community_id WHERE sc.survey_id=?1 AND m.user_id=?2)").bind(id).bind(user).fetch_one(&state.db).await?;
    if row.creator_id != user && !can_respond {
        return Err(AppError::forbidden());
    }
    let communities = sqlx::query_as("SELECT c.id,c.slug,c.name FROM communities c JOIN survey_communities sc ON sc.community_id=c.id WHERE sc.survey_id=?1 ORDER BY c.name").bind(id).fetch_all(&state.db).await?;
    let audience_count = sqlx::query_scalar("SELECT COUNT(DISTINCT m.user_id) FROM community_members m JOIN survey_communities sc ON sc.community_id=m.community_id WHERE sc.survey_id=?1").bind(id).fetch_one(&state.db).await?;
    let response_count =
        sqlx::query_scalar("SELECT COUNT(*) FROM survey_responses WHERE survey_id=?1")
            .bind(id)
            .fetch_one(&state.db)
            .await?;
    let answers: Option<String> = sqlx::query_scalar(
        "SELECT answers_json FROM survey_responses WHERE survey_id=?1 AND user_id=?2",
    )
    .bind(id)
    .bind(user)
    .fetch_optional(&state.db)
    .await?;
    let questions: Vec<Question> =
        serde_json::from_str(&row.questions_json).map_err(AppError::internal)?;
    let results = if row.creator_id == user && include_results {
        let mut results = Vec::new();
        for (index, question) in questions.iter().enumerate() {
            let mut result = QuestionResults {
                counts: Default::default(),
                texts: Vec::new(),
            };
            match question.kind {
                QuestionKind::SingleChoice => {
                    let counts: Vec<(String,i64)> = sqlx::query_as("SELECT json_extract(answers_json,?2), COUNT(*) FROM survey_responses WHERE survey_id=?1 GROUP BY json_extract(answers_json,?2)")
                        .bind(id).bind(format!("$[{index}]")).fetch_all(&state.db).await?;
                    result.counts = counts.into_iter().collect();
                }
                QuestionKind::ShortText => {
                    result.texts = sqlx::query_scalar("SELECT json_extract(answers_json,?2) FROM survey_responses WHERE survey_id=?1 ORDER BY created_at DESC,user_id LIMIT 50")
                        .bind(id).bind(format!("$[{index}]")).fetch_all(&state.db).await?;
                }
            }
            results.push(result);
        }
        Some(results)
    } else {
        None
    };
    Ok(Survey {
        id: row.id,
        creator_id: row.creator_id,
        title: row.title,
        description: row.description,
        created_at: row.created_at,
        questions,
        communities,
        audience_count,
        response_count,
        can_respond,
        answers: answers
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(AppError::internal)?,
        results,
    })
}
pub async fn get(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Survey>> {
    Ok(Json(read(&state, id, auth.id, true).await?))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    answers: Vec<String>,
}
pub async fn respond(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(input): Json<Response>,
) -> Result<(StatusCode, Json<Survey>)> {
    let survey = read(&state, id, auth.id, false).await?;
    if !survey.can_respond {
        return Err(AppError::forbidden());
    }
    if input.answers.len() != survey.questions.len() {
        return Err(AppError::bad_request("answer every question"));
    }
    let mut answers = Vec::new();
    for (q, answer) in survey.questions.iter().zip(input.answers) {
        let answer = validation::text(&answer, 4000)?;
        if matches!(q.kind, QuestionKind::SingleChoice) && !q.options.contains(&answer) {
            return Err(AppError::bad_request("select one of the offered options"));
        }
        answers.push(answer);
    }
    sqlx::query("INSERT INTO survey_responses (survey_id,user_id,answers_json) VALUES (?1,?2,?3)")
        .bind(id)
        .bind(auth.id)
        .bind(serde_json::to_string(&answers).map_err(AppError::internal)?)
        .execute(&state.db)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(read(&state, id, auth.id, true).await?),
    ))
}
