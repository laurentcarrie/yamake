// use crate::error as E;
use colored_text::Colorize;
use log;
use petgraph::Direction::Incoming;
use petgraph::dot::Dot;
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use petgraph::graph::NodeIndex;
use std::result::Result;
// use std::time::Duration;

use crate::model as M;
use crate::model::PathWithTag;
// use tokio::sync::mpsc::Receiver;

pub async fn expand(_g: &mut M::G) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(false)
}
