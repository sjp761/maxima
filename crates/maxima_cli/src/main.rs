use log::{error, info};
use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use maxima_proto::{comm::client::ProtoConnectionManager, comp::{ClientComponentManager, auth::{CheckAuthRequest, LoginRequest}}, entry::client_setup::setup_client};
use inquire::{Select, Text};

#[derive(Subcommand, Debug)]
enum Mode {
    Launch {
        slug: String,

        #[arg(long)]
        game_path: Option<String>,

        #[arg(long)]
        game_args: Vec<String>,

        /// When set, offer_id must be a content ID, and the only authenticated
        /// requests are made to the license server. A dummy name will be used
        /// in place of your real username, and any online LSX requests will fail
        #[arg(long)]
        login: Option<String>,
    },
    ListGames,
    LocateGame {
        path: String,
    },
    CloudSync {
        game_slug: String,

        #[arg(long)]
        write: bool,
    },
    AccountInfo,
    CreateAuthCode {
        #[arg(long)]
        client_id: String,
    },
    JunoTokenRefresh,
    ReadLicenseFile {
        #[arg(long)]
        content_id: String,
    },
    GetUserById {
        #[arg(long)]
        user_id: String,
    },
    GetGameBySlug {
        #[arg(long)]
        slug: String,
    },
    TestRTMConnection,
    ListFriends,
    GetLegacyCatalogDef {
        #[arg(long)]
        offer_id: String,
    },
    DownloadSpecificFile {
        #[arg(long)]
        offer_id: String,

        #[arg(long)]
        build_id: String,

        #[arg(long)]
        file: String,
    },
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    mode: Option<Mode>,

    #[arg(long)]
    #[clap(global = true)]
    login: Option<String>,
}

async fn print_account_info(comp_man: ClientComponentManager) -> Result<()> {
    let user = comp_man.users().local_user(()).await?;

    info!("Access Token: {}", comp_man.auth().access_token(()).await?);
    // TODO: info!("PC Sign: {}", AuthContext::new()?.generate_pc_sign()?);

    info!("Username: {}", user.display_name());
    info!("User ID: {}", user.account_id());
    info!("Persona ID: {}", user.persona_id());
    Ok(())
}

async fn run_interactive(comp_man: ClientComponentManager) -> Result<()> {
    let launch_options = vec![
        "Launch Game",
        "Install Game",
        "List Builds",
        "List Games",
        "Account Info",
    ];
    let name = Select::new(
        "Welcome to Maxima! What would you like to do?",
        launch_options,
    )
    .prompt()?;

    let _ = match name {
        "Launch Game" => unimplemented!("interactive_start_game"),
        "Install Game" => unimplemented!("interactive_install_game"),
        "List Builds" => unimplemented!("generate_download_links"),
        "List Games" => unimplemented!("list_games"),
        "Account Info" => print_account_info(comp_man).await,
        _ => bail!("Something went wrong."),
    };

    Ok(())
}

async fn startup() -> Result<()> {
    let args = Args::parse();

    
    let (conn_man, comp_man) = setup_client().await;

    let req = CheckAuthRequest::builder().allow_cached(false).build();
    let logged_in = comp_man.auth().check(req).await;
    if !logged_in.unwrap() {
        let req = LoginRequest::builder().build();
        let _ = comp_man.auth().login(req).await;
    }
    if args.mode.is_none() {
        run_interactive(comp_man).await?;
        return Ok(());
    }

    let mode = args.mode.unwrap();
    match mode {
        Mode::Launch { slug, game_path, game_args, login } => unimplemented!("launch_game"),
        Mode::ListGames => unimplemented!("list_games"),
        Mode::LocateGame { path } => unimplemented!("locate_game"),
        Mode::CloudSync { game_slug, write } => unimplemented!("cloud_sync"),
        Mode::AccountInfo => print_account_info(comp_man).await?,
        Mode::CreateAuthCode { client_id } => unimplemented!("create_auth_code"),
        Mode::JunoTokenRefresh => unimplemented!("juno_token_refresh"),
        Mode::ReadLicenseFile { content_id } => unimplemented!("read_license_file"),
        Mode::ListFriends => unimplemented!("list_friends"),
        Mode::GetUserById { user_id } => unimplemented!("get_user_by_id"),
        Mode::GetGameBySlug { slug } => unimplemented!("get_game_by_slug"),
        Mode::TestRTMConnection => unimplemented!("test_rtm_connection"),
        Mode::GetLegacyCatalogDef { offer_id } => unimplemented!("get_legacy_catalog_def"),
        Mode::DownloadSpecificFile { offer_id, build_id, file } => unimplemented!("download_specific_file"),
    };

    Ok(())
}



#[tokio::main]
async fn main() {
    let result = startup().await;

    if let Some(e) = result.err() {
        match std::env::var("RUST_BACKTRACE") {
            Ok(_) => error!("{}:\n{}", e, e.backtrace().to_string()),
            Err(_) => error!("{}: {}", e, e.root_cause()),
        }
    }
}