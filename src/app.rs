use crate::types::{
    Comment, StoryItem, StoryPageData, StorySorting, UserData,
};
pub use content::Content;
use sauron::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use crate::api;
use crate::api::ServerError;

mod content;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FetchStatus<T> {
    Idle,
    Loading,
    Complete(T),
    Error(String),
}

pub enum Msg {
    FetchStories,
    FetchStoriesSorted(StorySorting),
    OpenStory(i64),
    /// show the user data of this username
    ShowUserPage(String),
    ShowCommentPermalink(i64),
    /// the new url and the Content
    ReceivedContent(Content),
    RequestError(ServerError),
    /// the new url
    UrlChanged(String),
    NoOp,
}

// App and all its members should be Serializable by serde
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    /// the content to be displayed in out app
    pub content: FetchStatus<Content>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            content: FetchStatus::Idle,
        }
    }
}

impl Application for App {

    type MSG = Msg;

    #[cfg(feature = "wasm")]
    fn init(&mut self) -> Cmd<Msg> {
        // Handle initial load by checking current hash/pathname
        let location = sauron::window().location();
        let hash = location.hash().unwrap_or_default();
        let pathname = location.pathname().unwrap_or_default();
        let url = if hash.is_empty() { pathname } else { hash.clone() };
        
        Cmd::batch([
            // Use popstate for now, we'll handle hash changes through URL monitoring
            Window::on_popstate(|_e| {
                log::trace!("popstate is triggered in sauron add event listener");
                let location = sauron::window().location();
                let hash = location.hash().unwrap_or_default();
                let pathname = location.pathname().unwrap_or_default();
                let url = if hash.is_empty() { pathname } else { hash };
                Msg::UrlChanged(url)
            }),
            // Handle initial routing - for SPAs with hash routing, empty hash means root
            if !hash.is_empty() {
                Cmd::new(async move { Msg::UrlChanged(url) })
            } else {
                // No hash means we're at the app root, fetch stories
                match self.content{
                    FetchStatus::Idle => {
                        self.fetch_stories()
                    }
                    _ => Cmd::none(),
                }
            },
        ])
    }

    fn view(&self) -> Node<Msg> {
        node! {
            <body class="main-layout">
                <header>
                   <a relative href="/"
                        on_click=|e|{
                            e.prevent_default();
                            Msg::FetchStories
                        }>
                       <div class="yc-logo">"Y"</div>
                   </a>
                   <a relative href="/"
                        on_click=|e|{
                            e.prevent_default();
                            Msg::FetchStories
                        }>
                        <h1>"Hacker News"</h1>
                   </a>
                   { self.view_story_sorting() }
                   <nav class="right-nav">
                        <a href="https://github.com/ivanceras/hackernews-sauron">
                           <svg role="img" xmlns="http://www.w3.org/2000/svg">
                              <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
                           </svg>
                        </a>
                   </nav>
                </header>
                    { self.view_loader() }
                <main class="content">
                    { self.view_content() }
                </main>
                <footer>
                    <hr/>
                    <nav class="repo-link">
                   "Powered by "<a href="https://github.com/ivanceras/sauron" target="_blank" rel="noopener noreferrer">"Sauron"</a>
                   </nav>
                </footer>
            </body>
        }
    }

    #[cfg(not(feature = "wasm"))]
    fn update(&mut self, _msg: Msg) -> Cmd<Msg> {
        Cmd::none()
    }

    #[cfg(feature = "wasm")]
    fn update(&mut self, msg: Msg) -> Cmd<Msg> {
        match msg {
            Msg::FetchStories => {
                Self::push_state_url("/");
                self.content = FetchStatus::Loading;
                self.fetch_stories()
            }
            Msg::FetchStoriesSorted(sorting) => {
                Self::push_state_url(&sorting.to_url());
                self.content = FetchStatus::Loading;
                self.fetch_stories_with_sorting(sorting)
            }
            Msg::OpenStory(story_id) => {
                Self::push_state_url(&StoryItem::to_url(story_id));
                self.content = FetchStatus::Loading;
                self.fetch_story_page(story_id)
            }
            Msg::ShowUserPage(username) => {
                Self::push_state_url(&UserData::to_url(&username));
                self.content = FetchStatus::Loading;
                log::trace!("showing user: {}", username);
                self.fetch_user_page(username)
            }
            Msg::ShowCommentPermalink(comment_id) => {
                Self::push_state_url(&Comment::to_url(comment_id));
                self.content = FetchStatus::Loading;
                log::trace!("showing comment: {}", comment_id);
                self.fetch_comment_permalink(comment_id)
            }
            Msg::ReceivedContent(content) => {
                self.content = FetchStatus::Complete(content);
                Cmd::from(Window::scroll_to_top(Msg::NoOp))
            }
            Msg::RequestError(server_error) => {
                log::error!("Error: {}", server_error);
                self.content = FetchStatus::Error(server_error.to_string());
                Cmd::none()
            }
            Msg::UrlChanged(url) => {
                self.content = FetchStatus::Loading;
                log::trace!("url changed to: {}", url);
                
                // Extract hash from URL - for hash routing, we only care about the hash part
                let hash = if let Some(hash_part) = url.split('#').nth(1) {
                    format!("#{}", hash_part)
                } else {
                    // If no hash, treat as root
                    String::new()
                };
                
                let cmd = if let Some(sorting) = StorySorting::from_url(&hash) {
                    self.fetch_stories_with_sorting(sorting)
                } else if let Some(story_id) = StoryItem::id_from_url(&hash) {
                    self.fetch_story_page(story_id)
                } else if let Some(comment_id) = Comment::id_from_url(&hash) {
                    self.fetch_comment_permalink(comment_id)
                } else if let Some(username) = UserData::id_from_url(&hash) {
                    self.fetch_user_page(username)
                } else if hash.is_empty() || hash == "#" {
                    // Empty hash or just "#" means root - fetch stories
                    self.fetch_stories()
                } else {
                    log::trace!("No appropriate route found for hash: {}", hash);
                    Cmd::none()
                };

                Cmd::batch(vec![
                    cmd,
                    Cmd::from(Window::scroll_to_top(Msg::NoOp)),
                ])
            }
            Msg::NoOp => Cmd::none(),
        }
    }
}

impl App {
    fn view_content(&self) -> Node<Msg> {
        match &self.content {
            FetchStatus::Idle => node! { <p>"Waiting around..."</p> },
            FetchStatus::Loading => node! { <p>"Loading..."</p> },
            FetchStatus::Error(e) => {
                node! {
                    <article>
                        <p>"Okay, something went wrong. I think it was: "</p>
                        <code>{text(e)}</code>
                    </article>
                }
            }
            FetchStatus::Complete(content) => content.view(),
        }
    }

    fn view_story_sorting(&self) -> Node<Msg>{
        nav([class("story-sort")],
            StorySorting::all().into_iter().map(|sorting|{
                a([href(format!("/{}",sorting.to_str())),
                    on_click(move|e|{
                        e.prevent_default();
                        Msg::FetchStoriesSorted(sorting)
                    })],
                    [text!("{}", sorting.to_str())]
                 )
            })
        )
    }

    fn view_loader(&self) -> Node<Msg> {
        node! {
            <div id="loader">
            {
                if matches!(self.content, FetchStatus::Loading) {
                    node!{
                        <div>
                            <div class="line"></div>
                            <div class="moving-ball"></div>
                        </div>
                    }
                }else{
                    node!{
                        <span></span>
                    }
                }
            }
            </div>
        }
    }
}

impl App {
    pub fn with_stories(stories: Vec<StoryItem>) -> Self {
        Self {
            content: FetchStatus::Complete(Content::from(stories)),
        }
    }
    pub fn with_story(story_page: StoryPageData) -> Self {
        Self {
            content: FetchStatus::Complete(Content::from(story_page)),
        }
    }
    pub fn with_user_page(user_data: UserData) -> Self {
        Self {
            content: FetchStatus::Complete(Content::from(user_data)),
        }
    }

    pub fn with_comment_permalink(comment: Comment) -> Self {
        Self {
            content: FetchStatus::Complete(Content::from(comment)),
        }
    }
}

#[cfg(feature = "wasm")]
impl App{

    fn fetch_stories(&self) -> Cmd<Msg> {
        Cmd::new( async move{
            match api::get_stories().await {
                Ok(stories) => {
                    Msg::ReceivedContent( Content::from(
                        stories,
                    ))
                }
                Err(e) => {
                    Msg::RequestError(e)
                }
            }
        })
    }

    fn fetch_stories_with_sorting(
        &self,
        sorting: StorySorting,
    ) -> Cmd<Msg> {
        Cmd::new( async move{
            match api::get_stories_with_sorting(sorting).await {
                Ok(stories) => {
                    Msg::ReceivedContent( Content::from(
                        stories,
                    ))
                }
                Err(e) => {
                    Msg::RequestError(e)
                }
            }
        })
    }


    fn fetch_story_page(&self, story_id: i64) -> Cmd<Msg> {
        Cmd::new( async move{
            match api::get_story(story_id).await {
                Ok(story) => {
                    Msg::ReceivedContent( Content::from(
                        story,
                    ))
                }
                Err(e) => {
                    Msg::RequestError(e)
                }
            }
        })
    }


    fn fetch_comment_permalink(&self, comment_id: i64) -> Cmd<Msg> {
        Cmd::new( async move{
            match api::get_comment(comment_id).await {
                Ok(comment) => {
                    Msg::ReceivedContent( Content::from(
                        comment,
                    ))
                }
                Err(e) => {
                    Msg::RequestError(e)
                }
            }
        })
    }


    fn fetch_user_page(&self, username: String) -> Cmd<Msg> {
        Cmd::new( async move{
            match api::get_user_page(&username).await {
                Ok(user_page) => {
                    Msg::ReceivedContent( Content::from(
                        user_page,
                    ))
                }
                Err(e) => {
                    Msg::RequestError(e)
                }
            }
        })
    }

    fn push_state_url(url: &str) {
        let window = sauron::window();
        let location = window.location();
        log::trace!("setting hash to: {}", url);
        
        // For hash routing, set the hash directly
        if url.starts_with('#') {
            location.set_hash(url).expect("must set hash");
        } else if url == "/" {
            location.set_hash("").expect("must set hash");
        } else {
            location.set_hash(&format!("#{}", url.trim_start_matches('/'))).expect("must set hash");
        }
    }
}


