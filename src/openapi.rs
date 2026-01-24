//! OpenAPI Documentation Definition
//! 
//! Contains the ApiDoc struct with all schema and path registrations.

use utoipa::OpenApi;

use crate::{
    models::{
        feed::{HomefeedRequest, HomefeedResponse, HomefeedData, HomefeedItem, NoteCard, NoteUser, NoteCover, CoverImageInfo, InteractInfo, NoteVideo, VideoCapa},
        search::{QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord, SearchRecommendResponse, SearchRecommendData, SugItem,
            SearchNotesRequest, SearchNotesResponse, SearchNotesData, SearchFilterOption,
            SearchOneboxRequest, SearchOneboxResponse,
            SearchFilterResponse, SearchFilterData, FilterItem, FilterTag,
            SearchUserRequest, SearchUserResponse, SearchUserData, SearchUserItem
        },
        user::{UserMeResponse, UserInfo},
    },
    api::notification::{
        mentions::{MentionsResponse, MentionsData},
        connections::{ConnectionsResponse, ConnectionsData},
        likes::{LikesResponse, LikesData},
    },
    api::login::{GuestInitResponse, CreateQrCodeResponse, PollStatusResponse, QrCodeStatusData, LoginInfo},
    api::note::detail::{NoteDetailRequest, NoteDetailResponse},
    api::media::{
        video::{VideoRequest, VideoResponse, VideoData, VideoItem},
        images::{ImagesRequest, ImagesResponse, ImagesData, ImageItem},
        download::{DownloadRequest, DownloadResponse, DownloadData},
    },
    handlers::search as search_handlers,
    handlers::auth as auth_handlers,
    handlers::notification as notification_handlers,
    handlers::user as user_handlers,

    handlers::media as media_handlers,
    handlers::creator as creator_handlers,
    api,
    api::creator::{
        models::{CreatorQrcodeCreateRequest, CreatorQrcodeStatusRequest}
    }
};

#[derive(OpenApi)]
#[openapi(
    paths(
        search_handlers::query_trending_handler,
        search_handlers::search_recommend_handler,
        search_handlers::search_notes_handler,
        search_handlers::search_onebox_handler,
        search_handlers::search_filter_handler,
        search_handlers::search_user_handler,
        user_handlers::user_me_handler,
        auth_handlers::guest_init_handler,
        auth_handlers::create_qrcode_handler,
        auth_handlers::poll_qrcode_status_handler,
        api::feed::category::get_category_feed,
        api::note::page::get_note_page,
        api::note::detail::get_note_detail,
        notification_handlers::mentions_handler,
        notification_handlers::connections_handler,
        notification_handlers::likes_handler,
        media_handlers::images_handler,
        media_handlers::download_handler,
        creator_handlers::creator_guest_init_handler,
        creator_handlers::creator_create_qrcode_handler,
        creator_handlers::creator_check_qrcode_status,
    ),
    components(
        schemas(
            GuestInitResponse, CreateQrCodeResponse, PollStatusResponse, QrCodeStatusData, LoginInfo,
            QueryTrendingResponse, QueryTrendingData, TrendingQuery, TrendingHintWord,
            SearchRecommendResponse, SearchRecommendData, SugItem,
            SearchNotesRequest, SearchNotesResponse, SearchNotesData, SearchFilterOption,
            SearchOneboxRequest, SearchOneboxResponse,
            SearchFilterResponse, SearchFilterData, FilterItem, FilterTag,
            SearchUserRequest, SearchUserResponse, SearchUserData, SearchUserItem,
            UserMeResponse, UserInfo,
            MentionsResponse, MentionsData,
            ConnectionsResponse, ConnectionsData,
            LikesResponse, LikesData,
            HomefeedRequest, HomefeedResponse, HomefeedData, HomefeedItem, NoteCard, NoteUser, NoteCover, CoverImageInfo, InteractInfo, NoteVideo, VideoCapa,
            NoteDetailRequest, NoteDetailResponse,
            VideoRequest, VideoResponse, VideoData, VideoItem,
            ImagesRequest, ImagesResponse, ImagesData, ImageItem,
            DownloadRequest, DownloadResponse, DownloadData,
            CreatorQrcodeCreateRequest, CreatorQrcodeStatusRequest
        )
    ),
    tags(
        (name = "xhs", description = "小红书 API 接口"),
        (name = "auth", description = "用户认证 (User Auth)"),
        (name = "Creator", description = "创作者中心认证 (Creator Auth)"),
        (name = "Feed", description = "主页发现频道：recommend(推荐)、fashion(穿搭)、food(美食)、cosmetics(彩妆)、movie_and_tv(影视)、career(职场)、love(情感)、household_product(家居)、gaming(游戏)、travel(旅行)、fitness(健身)"),
        (name = "Note", description = "笔记相关接口：detail(详情)、page(评论)、video(视频地址)"),
        (name = "Media", description = "媒体文件操作：video(视频地址解析)、images(图片地址解析)、download(通用媒体下载)"),
        (name = "Search", description = "搜索相关接口：notes(笔记)、usersearch(用户)、onebox(聚合)、recommend(推荐)、filter(筛选)")
    )
)]
pub struct ApiDoc;

