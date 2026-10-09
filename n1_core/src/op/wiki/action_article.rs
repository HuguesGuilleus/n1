use n1_tool::DB;
use serde::Deserialize;

use crate::{
    DTO, OpRequest, Result, errs,
    op::{OID_ENTITY_WIKI, wiki::Article},
};

#[derive(Debug, Deserialize)]
pub struct ArticleIdsDTO {
    pub owner_id: u32,
    pub article_id: u32,
}
impl DTO for ArticleIdsDTO {
    fn check(&self) -> Result<()> {
        if self.owner_id == 0 {
            errs::FIELD_ID_ZERO.push_result("field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID_ZERO.push_result("field `article_id`")?;
        }
        Ok(())
    }
}
pub async fn article_get(db: &impl DB, r: OpRequest<ArticleIdsDTO>) -> Result<Article> {
    let article: Article = db.obj_get(r.dto.owner_id, r.dto.article_id).await?;

    if article.oid == 0 {
        errs::NOT_FOUND.push_result("article not found")?;
    }

    Ok(article)
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
            errs::FIELD_ID_ZERO.push_result("the integer field `owner_id`")?;
        } else if self.title.trim().is_empty() {
            errs::FIELD_EMPTY.push_result("the string field `title`")?;
        } else if self.slug.trim().is_empty() {
            errs::FIELD_EMPTY.push_result("the string field `slug`")?;
        }
        Ok(())
    }
}
pub async fn article_new(db: &impl DB, r: OpRequest<NewArticleDTO>) -> Result<u32> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let oid = db.fs_new_object(r.dto.owner_id).await?;
    let article = Article {
        oid,
        slug: r.dto.slug,
        title: r.dto.title,
        last_edit: db.now()?,
        content: "...".to_string(),
    };

    db.obj_set(r.dto.owner_id, oid, article).await?;

    // todo: regenerate wiki

    Ok(oid)
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
            errs::FIELD_ID_ZERO.push_result("the integer field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID_ZERO.push_result("the integer field `article_id`")?;
        } else if self.title.trim().is_empty() {
            errs::FIELD_EMPTY.push_result("the string field `title`")?;
        }
        Ok(())
    }
}
pub async fn article_set_title(db: &impl DB, r: OpRequest<ArticleSetTitleDTO>) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let mut article: Article = db.obj_get(r.dto.owner_id, r.dto.article_id).await?;
    article.title = r.dto.title;

    db.obj_set(r.dto.owner_id, r.dto.article_id, &article)
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
            errs::FIELD_ID_ZERO.push_result("the integer field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID_ZERO.push_result("the integer field `page_article_idid`")?;
        } else if self.slug.trim().is_empty() {
            errs::FIELD_EMPTY.push_result("the string field `slug`")?;
        }
        Ok(())
    }
}
pub async fn article_set_slug(db: &impl DB, r: OpRequest<ArticleSetSlugDTO>) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let mut article: Article = db.obj_get(r.dto.owner_id, r.dto.article_id).await?;
    article.slug = r.dto.slug;

    db.obj_set(r.dto.owner_id, r.dto.article_id, &article)
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
            errs::FIELD_ID_ZERO.push_result("the integer field `owner_id`")?;
        } else if self.article_id == 0 {
            errs::FIELD_ID_ZERO.push_result("the integer field `article_id`")?;
        } else if self.content.trim().is_empty() {
            errs::FIELD_EMPTY.push_result("the string field `content`")?;
        }
        Ok(())
    }
}
pub async fn article_set_content(db: &impl DB, r: OpRequest<ArticleSetContentDTO>) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    let mut article: Article = db.obj_get(r.dto.owner_id, r.dto.article_id).await?;
    article.content = r.dto.content;

    db.obj_set(r.dto.owner_id, r.dto.article_id, &article)
        .await?;

    Ok(())
}

pub async fn article_rm(db: &impl DB, r: OpRequest<ArticleIdsDTO>) -> Result<()> {
    r.token
        .check_access_write(r.dto.owner_id, OID_ENTITY_WIKI)?;

    db.fs_rm_object(r.dto.owner_id, r.dto.article_id).await?;

    Ok(())
}
