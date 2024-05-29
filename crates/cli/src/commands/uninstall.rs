use crate::commands::clean::purge_tool;
use crate::helpers::{create_progress_bar, disable_progress_bars, ProtoResource};
use crate::telemetry::{track_usage, Metric};
use clap::Args;
use proto_core::{Id, ProtoConfig, Tool, UnresolvedVersionSpec};
use starbase::system;
use std::process;
use tracing::debug;

#[derive(Args, Clone, Debug)]
pub struct UninstallArgs {
    #[arg(required = true, help = "ID of tool")]
    id: Id,

    #[arg(help = "Version or alias of tool")]
    spec: Option<UnresolvedVersionSpec>,

    #[arg(long, help = "Avoid and force confirm prompts")]
    yes: bool,
}

fn unpin_version(args: &UninstallArgs, proto: &ProtoResource) -> miette::Result<()> {
    let manager = proto.env.load_config_manager()?;

    for file in &manager.files {
        if !file.exists {
            continue;
        }

        ProtoConfig::update(&file.path, |config| {
            if let Some(versions) = &mut config.versions {
                let remove = if let Some(version) = versions.get(&args.id) {
                    args.spec.is_none() || args.spec.as_ref().is_some_and(|spec| spec == version)
                } else {
                    false
                };

                if remove {
                    versions.remove(&args.id);
                }
            }
        })?;
    }

    Ok(())
}

#[system]
pub async fn uninstall(args: ArgsRef<UninstallArgs>, proto: ResourceRef<ProtoResource>) {
    // Uninstall everything
    let Some(spec) = &args.spec else {
        let tool = purge_tool(proto, &args.id, args.yes).await?;

        unpin_version(&args, proto)?;

        // Track usage metrics
        track_uninstall(&tool, true).await?;

        return Ok(());
    };

    // Uninstall a tool by version
    let mut tool = proto.load_tool(&args.id).await?;

    if !tool.is_setup(spec).await? {
        eprintln!(
            "{} {} has not been installed locally",
            tool.get_name(),
            tool.get_resolved_version(),
        );

        process::exit(1);
    }

    debug!("Uninstalling {} with version {}", tool.get_name(), spec);

    if tool.disable_progress_bars() {
        disable_progress_bars();
    }

    let pb = create_progress_bar(format!(
        "Uninstalling {} {}",
        tool.get_name(),
        tool.get_resolved_version()
    ));

    let uninstalled = tool.teardown().await?;

    unpin_version(&args, proto)?;

    pb.finish_and_clear();

    if !uninstalled {
        return Ok(());
    }

    // Track usage metrics
    track_uninstall(&tool, false).await?;

    println!(
        "{} {} has been uninstalled!",
        tool.get_name(),
        tool.get_resolved_version(),
    );
}

async fn track_uninstall(tool: &Tool, purged: bool) -> miette::Result<()> {
    track_usage(
        &tool.proto,
        Metric::UninstallTool {
            id: tool.id.to_string(),
            plugin: tool
                .locator
                .as_ref()
                .map(|loc| loc.to_string())
                .unwrap_or_default(),
            version: if purged {
                "*".into()
            } else {
                tool.get_resolved_version().to_string()
            },
        },
    )
    .await
}
