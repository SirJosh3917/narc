use serenity::{model::id::*, utils::Colour};

#[derive(Debug, Clone)]
pub struct ReportModel {
    pub id: u64,
    pub accuser_user_id: UserId,
    pub reported_user_id: UserId,
    pub guild_id: GuildId,
    pub status: ReportStatus,
    pub channel_id: Option<ChannelId>,
    pub message_id: Option<MessageId>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ViewModel {
    User(UserViewModel),
    Mod(ModViewModel),
}

#[derive(Debug, Clone)]
pub struct UserViewModel {
    pub report_id: u64,
    pub message_id: MessageId,
    pub status: ReportStatus,
}

#[derive(Debug, Clone)]
pub struct ModViewModel {
    pub report_id: u64,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub preview_archive_id: u64,
    pub handler: Option<UserId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReportStatus {
    Unhandled,
    Reviewing,
    Accepted,
    Denied,
}

impl ReportModel {
    pub fn url(&self) -> Option<String> {
        self.message_id.and_then(|m| {
            self.channel_id
                .map(|c| format!("https://discord.com/channels/{}/{}/{}", self.guild_id, c, m))
        })
    }
}

impl From<i64> for ReportStatus {
    fn from(value: i64) -> Self {
        match value {
            0 => Self::Unhandled,
            1 => Self::Reviewing,
            2 => Self::Accepted,
            3 => Self::Denied,
            _ => panic!("unknown ReportStatus {}", value),
        }
    }
}

impl Into<i64> for ReportStatus {
    fn into(self) -> i64 {
        match self {
            Self::Unhandled => 0,
            Self::Reviewing => 1,
            Self::Accepted => 2,
            Self::Denied => 3,
        }
    }
}

impl ReportStatus {
    pub fn to_human_status(&self) -> &str {
        match *self {
            ReportStatus::Unhandled => "😴 Unhandled",
            ReportStatus::Reviewing => "🔎 Reviewing",
            ReportStatus::Accepted => "✅ Accepted",
            ReportStatus::Denied => "❌ Denied",
        }
    }

    pub fn to_color(&self) -> Option<Colour> {
        Some(Colour::new(match self {
            ReportStatus::Unhandled => return None,
            ReportStatus::Reviewing => 0xADD8E6,
            ReportStatus::Denied => 0xFF0000,
            ReportStatus::Accepted => 0x00FF00,
        }))
    }
}
