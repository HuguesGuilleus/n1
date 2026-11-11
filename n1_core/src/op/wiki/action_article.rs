use bytes::Bytes;
use n1_tool::DB;
use serde::{Deserialize, Serialize};

use crate::{
    DTO, ErrorStep, OpRequest, Result,
    op::{
        Name, OID_ENTITY_WIKI,
        wiki::{
            WikiArticle,
            render_pub::{render_pub_article, render_pub_index},
        },
    },
};

#[derive(Debug, Deserialize)]
pub struct ArticleIdsDTO {
    pub owner: u32,
    pub article: u32,
}
impl DTO for ArticleIdsDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            ErrorStep::FieldZero { name: "owner" }.result()?
        } else if self.article == 0 {
            ErrorStep::FieldZero { name: "article" }.result()?
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Default, Clone)]
pub struct ArticleOutDTO {
    // Page id
    pub id: u32,
    // page end path. Do not contain spaces.
    pub slug: Name,
    // Display title
    pub title: String,
    /// Last edition, in second since Epoch.
    pub last_edit: u64,
    /// The page content.
    pub content: String,
}
/// Get the article including his content.
pub async fn article_get(db: &impl DB, r: OpRequest<ArticleIdsDTO>) -> Result<ArticleOutDTO> {
    let mutex = r.common.wiki.get(r.dto.owner).await?;
    let wiki = mutex.read().await;
    let article = wiki
        .articles
        .get(&r.dto.article)
        .ok_or(ErrorStep::NotFoundObject {
            owner: r.dto.owner,
            object: r.dto.article,
            type_name: "article",
        })?;

    let content =
        String::from_utf8_lossy(&db.fs_get(r.dto.owner, r.dto.article).await?).to_string();

    Ok({
        ArticleOutDTO {
            id: article.id,
            slug: article.slug.clone(),
            title: article.title.clone(),
            last_edit: article.last_edit,
            content,
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct NewArticleDTO {
    pub owner: u32,
    pub title: String,
    pub slug: Name,
    pub content: String,
}
impl DTO for NewArticleDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            ErrorStep::FieldZero { name: "owner_id" }.result()?
        } else if self.title.trim().is_empty() {
            ErrorStep::FieldEmpty { name: "title" }.result()?
        } else if self.content.is_empty() {
            ErrorStep::FieldEmpty { name: "content" }.result()?
        }
        // slug not verified because it is of type `Name` which already enforces its own constraints.
        Ok(())
    }
}
/// Create a new article. Returns the ID of the newly created article.
pub async fn article_new(db: &impl DB, r: OpRequest<NewArticleDTO>) -> Result<u32> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let article_id = db.fs_new_object(r.dto.owner).await?;
    let mutex = r.common.wiki.get(r.dto.owner).await?;
    let mut wiki = mutex.write().await;
    let article = WikiArticle {
        id: article_id,
        slug: r.dto.slug,
        title: r.dto.title,
        last_edit: db.now()?,
    };

    render_pub_article(db, r.common.lang, &owner, &article, &r.dto.content).await?;
    wiki.articles.insert(article_id, article);
    render_pub_index(db, &r.common, &owner, &wiki).await?;

    db.obj_set(r.dto.owner, OID_ENTITY_WIKI as u32, &*wiki)
        .await?;
    db.fs_set(r.dto.owner, article_id, Bytes::from(r.dto.content))
        .await?;

    Ok(article_id)
}

#[derive(Debug, Deserialize)]
pub struct ArticleSetTitleDTO {
    pub owner: u32,
    pub article: u32,
    pub title: String,
}
impl DTO for ArticleSetTitleDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            ErrorStep::FieldZero { name: "owner" }.result()?;
        } else if self.article == 0 {
            ErrorStep::FieldZero { name: "article" }.result()?;
        } else if self.title.trim().is_empty() {
            ErrorStep::FieldEmpty { name: "title" }.result()?;
        }
        Ok(())
    }
}
/// Set the title of a wiki article.
pub async fn article_set_title(db: &impl DB, r: OpRequest<ArticleSetTitleDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let mutex = r.common.wiki.get(r.dto.owner).await?;
    let mut wiki = mutex.write().await;
    let article = wiki
        .articles
        .get_mut(&r.dto.article)
        .ok_or(ErrorStep::NotFoundObject {
            owner: r.dto.owner,
            object: r.dto.article,
            type_name: "article",
        })?;
    article.title = r.dto.title;
    article.last_edit = db.now()?;

    let content =
        String::from_utf8_lossy(&db.fs_get(r.dto.owner, r.dto.article).await?).to_string();
    render_pub_article(db, r.common.lang, &owner, &article, &content).await?;
    render_pub_index(db, &r.common, &owner, &wiki).await?;

    db.obj_set(r.dto.owner, OID_ENTITY_WIKI as u32, &*wiki)
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ArticleSetSlugDTO {
    pub owner: u32,
    pub article: u32,
    pub slug: Name,
}
impl DTO for ArticleSetSlugDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            ErrorStep::FieldZero { name: "owner" }.result()?;
        } else if self.article == 0 {
            ErrorStep::FieldZero { name: "article" }.result()?;
        }
        // slug not verified because it is of type `Name` which already enforces its own constraints.
        Ok(())
    }
}
/// Set the slug (end path) of a wiki article.
pub async fn article_set_slug(db: &impl DB, r: OpRequest<ArticleSetSlugDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let mutex = r.common.wiki.get(r.dto.owner).await?;
    let mut wiki = mutex.write().await;
    let article = wiki
        .articles
        .get_mut(&r.dto.article)
        .ok_or(ErrorStep::NotFoundObject {
            owner: r.dto.owner,
            object: r.dto.article,
            type_name: "article",
        })?;
    article.slug = r.dto.slug;
    article.last_edit = db.now()?;

    let content =
        String::from_utf8_lossy(&db.fs_get(r.dto.owner, r.dto.article).await?).to_string();
    db.page_del(&format!(
        "/@{}/w/{}-{}",
        owner.name(),
        r.dto.article,
        article.slug
    ))
    .await?;
    render_pub_article(db, r.common.lang, &owner, &article, &content).await?;
    render_pub_index(db, &r.common, &owner, &wiki).await?;

    db.obj_set(r.dto.owner, OID_ENTITY_WIKI as u32, &*wiki)
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct ArticleSetContentDTO {
    pub owner: u32,
    pub article: u32,
    pub content: String,
}
impl DTO for ArticleSetContentDTO {
    fn check(&self) -> Result<()> {
        if self.owner == 0 {
            ErrorStep::FieldZero { name: "owner" }.result()?;
        } else if self.article == 0 {
            ErrorStep::FieldZero { name: "article" }.result()?;
        } else if self.content.trim().is_empty() {
            ErrorStep::FieldEmpty { name: "content" }.result()?;
        }
        Ok(())
    }
}
/// Set the content (plain text) of a wiki article.
pub async fn article_set_content(db: &impl DB, r: OpRequest<ArticleSetContentDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let mutex = r.common.wiki.get(r.dto.owner).await?;
    let mut wiki = mutex.write().await;
    let article = wiki
        .articles
        .get_mut(&r.dto.article)
        .ok_or(ErrorStep::NotFoundObject {
            owner: r.dto.owner,
            object: r.dto.article,
            type_name: "article",
        })?;
    article.last_edit = db.now()?;

    render_pub_article(db, r.common.lang, &owner, &article, &r.dto.content).await?;
    render_pub_index(db, &r.common, &owner, &wiki).await?;

    db.obj_set(r.dto.owner, OID_ENTITY_WIKI as u32, &*wiki)
        .await?;
    db.fs_set(r.dto.owner, r.dto.article, Bytes::from_owner(r.dto.content))
        .await?;

    Ok(())
}

/// Remove a wiki article.
pub async fn article_rm(db: &impl DB, r: OpRequest<ArticleIdsDTO>) -> Result<()> {
    r.token.check_access_write(r.dto.owner, OID_ENTITY_WIKI)?;

    let owner = r.common.get_entity(r.dto.owner).await?;
    let mutex = r.common.wiki.get(r.dto.owner).await?;
    let mut wiki = mutex.write().await;
    let article = match wiki.articles.remove(&r.dto.article) {
        Some(article) => article,
        None => {
            return Err(ErrorStep::NotFoundObject {
                owner: r.dto.owner,
                object: r.dto.article,
                type_name: "article",
            }
            .into());
        }
    };

    db.page_del(&format!(
        "/@{}/w/{}-{}",
        owner.name(),
        r.dto.article,
        article.slug
    ))
    .await?;
    render_pub_index(db, &r.common, &owner, &wiki).await?;

    db.obj_set(r.dto.owner, OID_ENTITY_WIKI as u32, &*wiki)
        .await?;
    db.fs_rm_object(r.dto.owner, r.dto.article).await?;

    Ok(())
}
