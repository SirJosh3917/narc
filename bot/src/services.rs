use crate::{
    database::{models::ReportStatus, Database, MakeReportEffect, ReportUpdateError},
    error_handling::handle_err_dms,
    view,
};
use serenity::{client::Context, model::id::*, prelude::Mentionable};
use thiserror::Error;

type ReportId = u64;

#[derive(Debug, Error)]
pub enum MakeReportError {
    #[error("Cannot submit report for unconfigured server")]
    UnconfiguredServer,
    #[error("An SQL error occured: {0}")]
    SqlError(#[from] sqlx::Error),
    #[error("A Discord error occured: {0}")]
    DiscordError(#[from] serenity::Error),
    #[error("An error occured while updating the view: {0}")]
    ViewError(#[from] view::UpdateViewError),
    #[error("An error occurred while updating the report: {0}")]
    UpdateError(#[from] ReportUpdateError),
}

pub async fn make_report(
    ctx: &Context,
    db: &Database,
    guild_id: GuildId,
    accuser_user_id: UserId,
    reported_user_id: UserId,
    reported_channel_id: Option<ChannelId>,
    reported_message_id: Option<MessageId>,
    report_reason: Option<&str>,
) -> Result<(), MakeReportError> {
    // before we make a report, lets ensure that the server is configured
    if !(db.has_server_config(&guild_id).await?) {
        return Err(MakeReportError::UnconfiguredServer);
    }

    if db.load_protected_user(&guild_id, &reported_user_id).await? {
        // don't return an error since we don't want public chat to get the error
        handle_err_dms(
            &ctx,
            accuser_user_id,
            None,
            &format!(
                "This user ({}) is protected on this server - you cannot report them",
                reported_user_id.mention()
            ),
            "Protected User",
        )
        .await;

        return Ok(());
    }

    let user_reporting = accuser_user_id.to_user(&ctx).await?;
    let reported_user = reported_user_id.to_user(&ctx).await?;

    let mut reported_message = None;

    if let Some(c) = reported_channel_id {
        if let Some(m) = reported_message_id {
            reported_message = Some(c.message(&ctx, m).await?);
        }
    }

    let effect = db
        .make_report(
            guild_id,
            &user_reporting,
            &reported_user,
            reported_message.as_ref(),
            report_reason,
        )
        .await?;

    if let MakeReportEffect::Duplicate(id) = effect {
        // don't return an error since we don't want public chat to get the error
        handle_err_dms(
            &ctx,
            user_reporting.id,
            None,
            &format!(
                "You have already submitted a report for this message (ID: {})",
                id
            ),
            "Duplicate Report",
        )
        .await;

        return Ok(());
    };

    view::update_report_view(ctx, &db, effect).await?;

    Ok(())
}

pub async fn update_report_reason(
    ctx: &Context,
    db: &Database,
    report_id: ReportId,
    reason: String,
) -> Result<(), MakeReportError> {
    db.update_report(report_id, Some(reason), None).await?;
    view::update_report_view(&ctx, &db, MakeReportEffect::Updated(report_id)).await?;
    Ok(())
}

pub async fn update_report_status(
    ctx: &Context,
    db: &Database,
    report_id: ReportId,
    status: ReportStatus,
) -> Result<(), MakeReportError> {
    db.update_report(report_id, Option::<String>::None, Some(status))
        .await?;
    view::update_report_view(&ctx, &db, MakeReportEffect::Updated(report_id)).await?;
    Ok(())
}
