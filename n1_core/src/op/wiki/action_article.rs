use n1_tool::Config;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, OpServer, Result, errs,
    op::{Json, OID_ENTITY_WIKI, wiki::Article},
};

#[derive(Debug, Deserialize)]
pub struct ArticleIdsDTO {
    pub owner_id: u32,
    pub article_id: u32,
}
impl DTO for ArticleIdsDTO {
    fn check(&self) -> Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID.push("field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_EMPTY.push("field `article_id`")?;
        }
        Ok(())
    }
}
pub async fn article_get(
    server: &OpServer<impl Config>,
    r: OpRequest<ArticleIdsDTO>,
) -> Result<Json<Article>> {
    let article: Article = server
        .config
        .obj_fetch(r.dto.owner_id, r.dto.article_id)
        .await?;

    if article.oid == 0 {
        errs::NOT_FOUND.push("article not found")?;
    }

    Ok(Json(article))
}

#[derive(Debug, Deserialize)]
pub struct NewArticleDTO {
    pub owner_id: u32,
    pub title: String,
    pub slug: String,
}
impl DTO for NewArticleDTO {
    fn check(&self) -> n1_tool::Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID.push("the integer field `owner_id`")?;
        } else if self.title.trim().is_empty() {
            errs::FIELD_EMPTY.push("the string field `title`")?;
        } else if self.slug.trim().is_empty() {
            errs::FIELD_EMPTY.push("the string field `slug`")?;
        }
        Ok(())
    }
}
pub async fn article_new(
    server: &OpServer<impl Config>,
    r: OpRequest<NewArticleDTO>,
) -> Result<Json<u32>> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let oid = server.config.fs_new_object(r.dto.owner_id).await?;
    let article = Article {
        oid,
        slug: r.dto.slug,
        title: r.dto.title,
        last_edit: server.config.now()?,
        content: "...".to_string(),
    };

    server
        .config
        .obj_store(r.dto.owner_id, oid, article)
        .await?;

    // todo: regenerate wiki

    Ok(Json(oid))
}

#[derive(Debug, Deserialize)]
pub struct ArticleSetTitleDTO {
    pub owner_id: u32,
    pub article_id: u32,
    pub title: String,
}
impl DTO for ArticleSetTitleDTO {
    fn check(&self) -> n1_tool::Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID.push("the integer field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID.push("the integer field `article_id`")?;
        } else if self.title.trim().is_empty() {
            errs::FIELD_EMPTY.push("the string field `title`")?;
        }
        Ok(())
    }
}
pub async fn article_set_title(
    server: &OpServer<impl Config>,
    r: OpRequest<ArticleSetTitleDTO>,
) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let mut article: Article = server
        .config
        .obj_fetch(r.dto.owner_id, r.dto.article_id)
        .await?;
    article.title = r.dto.title;

    server
        .config
        .obj_store(r.dto.owner_id, r.dto.article_id, &article)
        .await?;

    // todo: regenerate wiki

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ArticleSetSlugDTO {
    pub owner_id: u32,
    pub article_id: u32,
    pub slug: String,
}
impl DTO for ArticleSetSlugDTO {
    fn check(&self) -> n1_tool::Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID.push("the integer field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID.push("the integer field `page_article_idid`")?;
        } else if self.slug.trim().is_empty() {
            errs::FIELD_EMPTY.push("the string field `slug`")?;
        }
        Ok(())
    }
}
pub async fn article_set_slug(
    server: &OpServer<impl Config>,
    r: OpRequest<ArticleSetSlugDTO>,
) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let mut article: Article = server
        .config
        .obj_fetch(r.dto.owner_id, r.dto.article_id)
        .await?;
    article.slug = r.dto.slug;

    server
        .config
        .obj_store(r.dto.owner_id, r.dto.article_id, &article)
        .await?;

    // todo: regenerate wiki

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ArticleSetContentDTO {
    pub owner_id: u32,
    pub article_id: u32,
    pub content: String,
}
impl DTO for ArticleSetContentDTO {
    fn check(&self) -> n1_tool::Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID.push("the integer field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID.push("the integer field `article_id`")?;
        } else if self.content.trim().is_empty() {
            errs::FIELD_EMPTY.push("the string field `content`")?;
        }
        Ok(())
    }
}
pub async fn article_set_content(
    server: &OpServer<impl Config>,
    r: OpRequest<ArticleSetContentDTO>,
) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let mut article: Article = server
        .config
        .obj_fetch(r.dto.owner_id, r.dto.article_id)
        .await?;
    article.content = r.dto.content;

    server
        .config
        .obj_store(r.dto.owner_id, r.dto.article_id, &article)
        .await?;

    Ok(())
}

pub async fn article_rm(server: &OpServer<impl Config>, r: OpRequest<ArticleIdsDTO>) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    server
        .config
        .fs_rm_object(r.dto.owner_id, r.dto.article_id)
        .await?;

    Ok(())
}
