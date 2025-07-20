use chrono::{DateTime, Utc};
use sauron::prelude::*;
use sauron::vdom::element;

/// Decode HTML entities in text content
fn decode_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&#x3D;", "=")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Sanitize HTML and convert to Sauron virtual DOM nodes
pub fn parse_html_to_nodes<MSG>(text: &str) -> Vec<Node<MSG>> {
    // Configure ammonia to allow code-related tags
    let sanitized = ammonia::Builder::default()
        .add_tags(&["code", "pre", "tt"])  // Add code formatting tags
        .clean(text)
        .to_string();
    
    // Simple HTML parser for common HN tags
    // This is a basic implementation - for full HTML parsing you'd want html5ever
    parse_simple_html(&sanitized)
}

fn parse_simple_html<MSG>(html: &str) -> Vec<Node<MSG>> {
    let mut nodes = Vec::new();
    let mut current_pos = 0;
    
    while current_pos < html.len() {
        if let Some(tag_start) = html[current_pos..].find('<') {
            let tag_start = current_pos + tag_start;
            
            // Add text before tag
            if tag_start > current_pos {
                let text_content = &html[current_pos..tag_start];
                if !text_content.trim().is_empty() {
                    nodes.push(text(decode_entities(text_content)));
                }
            }
            
            // Find tag end
            if let Some(tag_end_pos) = html[tag_start..].find('>') {
                let tag_end = tag_start + tag_end_pos + 1;
                let tag_content = &html[tag_start..tag_end];
                
                if tag_content.starts_with("<p>") || tag_content.starts_with("<p ") {
                    // Handle paragraph - find closing tag
                    if let Some(close_pos) = html[tag_end..].find("</p>") {
                        let p_content = &html[tag_end..tag_end + close_pos];
                        nodes.push(element("p", [], parse_simple_html(p_content)));
                        current_pos = tag_end + close_pos + 4; // Skip "</p>"
                    } else {
                        // No closing tag, treat as line break
                        nodes.push(element("br", [], []));
                        current_pos = tag_end;
                    }
                } else if tag_content.starts_with("<i>") {
                    if let Some(close_pos) = html[tag_end..].find("</i>") {
                        let i_content = &html[tag_end..tag_end + close_pos];
                        nodes.push(element("i", [], parse_simple_html(i_content)));
                        current_pos = tag_end + close_pos + 4; // Skip "</i>"
                    } else {
                        current_pos = tag_end;
                    }
                } else if tag_content.starts_with("<b>") {
                    if let Some(close_pos) = html[tag_end..].find("</b>") {
                        let b_content = &html[tag_end..tag_end + close_pos];
                        nodes.push(element("b", [], parse_simple_html(b_content)));
                        current_pos = tag_end + close_pos + 4; // Skip "</b>"
                    } else {
                        current_pos = tag_end;
                    }
                } else if tag_content.starts_with("<br") {
                    nodes.push(element("br", [], []));
                    current_pos = tag_end;
                } else if tag_content.starts_with("<code>") {
                    if let Some(close_pos) = html[tag_end..].find("</code>") {
                        let code_content = &html[tag_end..tag_end + close_pos];
                        // Don't parse code content as HTML - treat as literal text
                        nodes.push(element("code", [], [text(decode_entities(code_content))]));
                        current_pos = tag_end + close_pos + 7; // Skip "</code>"
                    } else {
                        current_pos = tag_end;
                    }
                } else if tag_content.starts_with("<pre>") {
                    // For <pre>, we need to find the matching </pre> while ignoring any tags inside
                    let mut pre_end = tag_end;
                    let mut pre_depth = 1;
                    
                    while pre_depth > 0 && pre_end < html.len() {
                        if let Some(next_tag) = html[pre_end..].find('<') {
                            pre_end += next_tag;
                            if html[pre_end..].starts_with("</pre>") {
                                pre_depth -= 1;
                                if pre_depth == 0 {
                                    let pre_content = &html[tag_end..pre_end];
                                    // Parse the content inside <pre> as HTML to handle nested <code> tags
                                    nodes.push(element("pre", [], parse_simple_html(pre_content)));
                                    current_pos = pre_end + 6; // Skip "</pre>"
                                    break;
                                }
                            } else if html[pre_end..].starts_with("<pre>") {
                                pre_depth += 1;
                            }
                            pre_end += 1;
                        } else {
                            // No more tags found, parse rest as content with potential HTML
                            let pre_content = &html[tag_end..];
                            nodes.push(element("pre", [], parse_simple_html(pre_content)));
                            current_pos = html.len();
                            break;
                        }
                    }
                    
                    if pre_depth > 0 {
                        // Unclosed pre tag, skip it
                        current_pos = tag_end;
                    }
                } else if tag_content.starts_with("<tt>") {
                    if let Some(close_pos) = html[tag_end..].find("</tt>") {
                        let tt_content = &html[tag_end..tag_end + close_pos];
                        // Don't parse tt content as HTML - treat as literal text
                        nodes.push(element("tt", [], [text(decode_entities(tt_content))]));
                        current_pos = tag_end + close_pos + 5; // Skip "</tt>"
                    } else {
                        current_pos = tag_end;
                    }
                } else {
                    // Skip unknown tags
                    current_pos = tag_end;
                }
            } else {
                // Malformed tag, treat as text
                nodes.push(text(decode_entities(&html[current_pos..])));
                break;
            }
        } else {
            // No more tags, add remaining text
            let remaining = &html[current_pos..];
            if !remaining.trim().is_empty() {
                nodes.push(text(decode_entities(remaining)));
            }
            break;
        }
    }
    
    nodes
}

/// Return the time ago for a date
pub fn time_ago(date: DateTime<Utc>) -> String {
    let now = Utc::now();

    const SECONDS_IN_MINUTE: f32 = 60.0;
    const SECONDS_IN_HOUR: f32 = SECONDS_IN_MINUTE * 60.0;
    const SECONDS_IN_DAY: f32 = SECONDS_IN_HOUR * 24.0;
    const SECONDS_IN_YEAR: f32 = SECONDS_IN_DAY * 365.0; // Ignore leap years for now

    let seconds = (now - date).num_seconds() as f32;
    if seconds < SECONDS_IN_MINUTE {
        let seconds = seconds.floor() as i32;
        if seconds < 2 {
            format!("{} second", seconds)
        } else {
            format!("{} seconds", seconds)
        }
    } else if seconds < SECONDS_IN_HOUR {
        let minutes = (seconds / SECONDS_IN_MINUTE).floor() as i32;
        if minutes < 2 {
            format!("{} minute", minutes)
        } else {
            format!("{} minutes", minutes)
        }
    } else if seconds < SECONDS_IN_DAY {
        let hours = (seconds / SECONDS_IN_HOUR).floor() as i32;
        if hours < 2 {
            format!("{} hour", hours)
        } else {
            format!("{} hours", hours)
        }
    } else if seconds < SECONDS_IN_YEAR {
        let days = (seconds / SECONDS_IN_DAY).floor() as i32;
        if days < 2 {
            format!("{} day", days)
        } else {
            format!("{} days", days)
        }
    } else {
        let years = (seconds / SECONDS_IN_YEAR).floor() as i32;
        if years < 2 {
            format!("{} year", years)
        } else {
            format!("{} years", years)
        }
    }
}
