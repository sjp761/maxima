use anyhow::{Ok, Result};
use egui::Context;
use log::{debug, error, info};
use maxima::{
    core::{
        service_layer::{ServiceGame, ServiceGameImagesRequestBuilder, SERVICE_REQUEST_GAMEIMAGES},
        LockedMaxima,
    },
    util::native::maxima_dir,
};
use std::{
    fs,
    sync::mpsc::Sender,
};

use crate::{
    bridge_thread::MaximaLibResponse, ui_image::UIImageCacheLoaderCommand,
};

async fn get_preferred_hero_image(images: &Option<ServiceGame>) -> Option<String> {
    if images.is_none() {
        return None;
    }

    let key_art = images.as_ref().unwrap().key_art();
    if key_art.is_none() {
        return None;
    }

    let key_art = key_art.as_ref().unwrap();

    debug!("{:?}", key_art);
    if let Some(img) = key_art.aspect_10x3_image() {
        return Some(img.path().clone());
    }

    if let Some(img) = key_art.aspect_2x1_image() {
        return Some(img.path().clone());
    }

    if let Some(img) = key_art.aspect_16x9_image() {
        return Some(img.path().clone());
    }

    None
}

async fn get_logo_image(images: &Option<ServiceGame>) -> Option<String> {
    if images.is_none() {
        return None;
    }

    let logo_set = images.as_ref().unwrap().primary_logo();
    if logo_set.is_none() {
        return None;
    }

    let largest_logo = logo_set.as_ref().unwrap().largest_image();
    if largest_logo.is_none() {
        return None;
    }

    Some(largest_logo.as_ref().unwrap().path().to_string())
}

pub async fn game_images_request(
    maxima_arc: LockedMaxima,
    slug: String,
    channel: Sender<UIImageCacheLoaderCommand>,
    ctx: &Context,
) -> Result<()> {
    let game_hero = maxima_dir()
        .unwrap()
        .join("cache/ui/images/")
        .join(&slug)
        .join("hero.jpg");
    let game_logo = maxima_dir()
        .unwrap()
        .join("cache/ui/images/")
        .join(&slug)
        .join("logo.png");
    let has_hero = fs::metadata(&game_hero).is_ok();
    let has_logo = fs::metadata(&game_logo).is_ok();
    let images: Option<ServiceGame> = // TODO: make it a result
        if !has_hero || !has_logo { //game hasn't been cached yet
            let maxima = maxima_arc.lock().await;
            maxima.service_layer()
            .request(SERVICE_REQUEST_GAMEIMAGES, ServiceGameImagesRequestBuilder::default()
            .should_fetch_context_image(!has_logo)
            .should_fetch_backdrop_images(!has_hero)
            .game_slug(slug.clone())
            .locale(maxima.locale().short_str().to_owned())
            .build()?).await?
        } else { None };

    if !has_hero {
        if let Some(hero) = get_preferred_hero_image(&images).await {
            channel.send(UIImageCacheLoaderCommand::ProvideRemote(crate::ui_image::UIImageType::Hero(slug.clone()), hero)).unwrap()
        }
    }

    if !has_logo {
        if let Some(logo) = get_logo_image(&images).await {
            channel.send(UIImageCacheLoaderCommand::ProvideRemote(crate::ui_image::UIImageType::Logo(slug), logo)).unwrap()
        } else {
            channel.send(UIImageCacheLoaderCommand::Stub(crate::ui_image::UIImageType::Logo(slug))).unwrap()
        }
    }
    Ok(())
}
