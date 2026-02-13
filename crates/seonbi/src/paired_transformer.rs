use crate::html::{normalize_text, HtmlEntity, HtmlTagStack};

#[derive(Clone)]
struct Unclosed<M> {
    m: M,
    buffer: Vec<HtmlEntity>,
}

pub struct PairedTransformer<M> {
    pub ignores_tag_stack: fn(&HtmlTagStack) -> bool,
    pub match_start: fn(&[M], &str) -> Option<(M, String, String, String)>,
    pub match_end: fn(&str) -> Option<(M, String, String, String)>,
    pub are_matches_paired: fn(&M, &M) -> bool,
    pub transform_pair: fn(&M, &M, Vec<HtmlEntity>) -> Vec<HtmlEntity>,
}

pub fn transform_pairs<M: Clone>(
    transformer: &PairedTransformer<M>,
    entities: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    iter(transformer, Vec::new(), normalize_text(entities))
}

fn iter<M: Clone>(
    t: &PairedTransformer<M>,
    stack: Vec<Unclosed<M>>,
    entities: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    if entities.is_empty() {
        return if stack.is_empty() {
            Vec::new()
        } else {
            unstack(stack)
        };
    }

    let mut rest = entities;
    let first = rest.remove(0);

    match first {
        HtmlEntity::Text { tag_stack, raw_text } => {
            let prev_matches: Vec<M> = stack.iter().map(|u| u.m.clone()).collect();
            let start_match = (t.match_start)(&prev_matches.into_iter().rev().collect::<Vec<_>>(), &raw_text);
            let end_match = (t.match_end)(&raw_text);

            match (start_match, end_match) {
                (Some(captured), None) => roll(t, stack, captured, tag_stack, rest),
                (None, Some(ref captured @ (ref m, _, _, _)))
                    if stack.iter().any(|u| (t.are_matches_paired)(&u.m, m)) =>
                {
                    unroll(t, stack, captured.clone(), tag_stack, rest)
                }
                (Some(ref captured @ (_, ref pre, _, _)), Some(ref captured2 @ (ref m2, ref pre2, _, _))) => {
                    if pre.len() >= pre2.len()
                        && stack.iter().any(|u| (t.are_matches_paired)(&u.m, m2))
                    {
                        unroll(t, stack, captured2.clone(), tag_stack, rest)
                    } else {
                        roll(t, stack, captured.clone(), tag_stack, rest)
                    }
                }
                (None, _) => {
                    if stack.is_empty() {
                        let mut out = vec![HtmlEntity::Text { tag_stack, raw_text }];
                        out.extend(iter(t, stack, rest));
                        out
                    } else {
                        let mut stack2 = stack;
                        if let Some(top) = stack2.first_mut() {
                            top.buffer.insert(0, HtmlEntity::Text { tag_stack, raw_text });
                        }
                        iter(t, stack2, rest)
                    }
                }
            }
        }
        other => {
            if stack.is_empty() {
                let mut out = vec![other];
                out.extend(iter(t, stack, rest));
                out
            } else {
                let mut stack2 = stack;
                if let Some(top) = stack2.first_mut() {
                    top.buffer.insert(0, other);
                }
                iter(t, stack2, rest)
            }
        }
    }
}

fn roll<M: Clone>(
    t: &PairedTransformer<M>,
    mut stack: Vec<Unclosed<M>>,
    captured: (M, String, String, String),
    tag_stack: HtmlTagStack,
    entities: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    let (start_match, pre, token, post) = captured;
    let mut next_entities = prepend_text(&tag_stack, &post, entities);
    next_entities = normalize_text(next_entities);

    if stack.is_empty() {
        let unclosed = Unclosed {
            m: start_match,
            buffer: vec![HtmlEntity::Text {
                tag_stack: tag_stack.clone(),
                raw_text: token,
            }],
        };
        let mut out = prepend_text(&tag_stack, &pre, Vec::new());
        out.extend(iter(t, vec![unclosed], next_entities));
        out
    } else {
        let unclosed = Unclosed {
            m: start_match,
            buffer: vec![HtmlEntity::Text {
                tag_stack: tag_stack.clone(),
                raw_text: token,
            }],
        };
        if let Some(top) = stack.first_mut() {
            let mut new_buf = prepend_text(&tag_stack, &pre, Vec::new());
            new_buf.append(&mut top.buffer);
            top.buffer = new_buf;
        }
        stack.insert(0, unclosed);
        iter(t, stack, next_entities)
    }
}

fn unroll<M: Clone>(
    t: &PairedTransformer<M>,
    stack: Vec<Unclosed<M>>,
    captured: (M, String, String, String),
    tag_stack: HtmlTagStack,
    entities: Vec<HtmlEntity>,
) -> Vec<HtmlEntity> {
    let (end_match, pre, token, post) = captured;
    let (prefix_stack, remain_stack) = find_pair(t, &end_match, stack);

    let (unrolled, next_stack) = if let Some(s) = remain_stack.first() {
        let start_match = s.m.clone();
        let mut buf = unstack_partial(prefix_stack);
        buf.extend(s.buffer.clone());
        buf = prepend_text(&tag_stack, &pre, buf);
        buf = prepend_text(&tag_stack, &token, buf);
        buf.reverse();

        let transformed = if buf.iter().any(|e| (t.ignores_tag_stack)(e.tag_stack())) {
            buf
        } else {
            (t.transform_pair)(&start_match, &end_match, buf)
        };

        (transformed, remain_stack[1..].to_vec())
    } else {
        (
            vec![HtmlEntity::Text {
                tag_stack: tag_stack.clone(),
                raw_text: format!("{pre}{token}"),
            }],
            Vec::new(),
        )
    };

    let remain_entities = prepend_text(&tag_stack, &post, entities);
    if next_stack.is_empty() {
        let mut out = unrolled;
        out.extend(iter(t, Vec::new(), remain_entities));
        out
    } else {
        let mut stack2 = next_stack;
        if let Some(top) = stack2.first_mut() {
            let mut rev = unrolled.clone();
            rev.reverse();
            rev.append(&mut top.buffer);
            top.buffer = rev;
        }
        iter(t, stack2, remain_entities)
    }
}

fn find_pair<M: Clone>(
    t: &PairedTransformer<M>,
    end_match: &M,
    stack: Vec<Unclosed<M>>,
) -> (Vec<Unclosed<M>>, Vec<Unclosed<M>>) {
    let mut prefix = Vec::new();
    let mut remaining = stack;

    while let Some(head) = remaining.first() {
        if (t.are_matches_paired)(end_match, &head.m) {
            break;
        }
        prefix.push(remaining.remove(0));
    }

    (prefix, remaining)
}

fn unstack_partial<M: Clone>(stack: Vec<Unclosed<M>>) -> Vec<HtmlEntity> {
    let mut out = Vec::new();
    for u in stack {
        out.extend(u.buffer);
    }
    out
}

fn unstack<M: Clone>(stack: Vec<Unclosed<M>>) -> Vec<HtmlEntity> {
    let mut out = unstack_partial(stack);
    out.reverse();
    out
}

fn prepend_text(tag_stack: &HtmlTagStack, text: &str, mut entities: Vec<HtmlEntity>) -> Vec<HtmlEntity> {
    if text.is_empty() {
        return entities;
    }
    entities.insert(
        0,
        HtmlEntity::Text {
            tag_stack: tag_stack.clone(),
            raw_text: text.to_string(),
        },
    );
    entities
}
