use super::{tag::HtmlTag, tag_stack::HtmlTagStack};

pub type HtmlRawAttrs = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HtmlEntity {
    StartTag { tag_stack: HtmlTagStack, tag: HtmlTag, raw_attributes: HtmlRawAttrs },
    EndTag { tag_stack: HtmlTagStack, tag: HtmlTag },
    Text { tag_stack: HtmlTagStack, raw_text: String },
    Cdata { tag_stack: HtmlTagStack, text: String },
    Comment { tag_stack: HtmlTagStack, comment: String },
}

impl HtmlEntity {
    pub fn tag_stack(&self) -> &HtmlTagStack {
        match self {
            HtmlEntity::StartTag { tag_stack, .. }
            | HtmlEntity::EndTag { tag_stack, .. }
            | HtmlEntity::Text { tag_stack, .. }
            | HtmlEntity::Cdata { tag_stack, .. }
            | HtmlEntity::Comment { tag_stack, .. } => tag_stack,
        }
    }

    pub fn with_tag_stack(self, new_stack: HtmlTagStack) -> Self {
        match self {
            HtmlEntity::StartTag { tag, raw_attributes, .. } => {
                HtmlEntity::StartTag { tag_stack: new_stack, tag, raw_attributes }
            }
            HtmlEntity::EndTag { tag, .. } => HtmlEntity::EndTag { tag_stack: new_stack, tag },
            HtmlEntity::Text { raw_text, .. } => {
                HtmlEntity::Text { tag_stack: new_stack, raw_text }
            }
            HtmlEntity::Cdata { text, .. } => HtmlEntity::Cdata { tag_stack: new_stack, text },
            HtmlEntity::Comment { comment, .. } => {
                HtmlEntity::Comment { tag_stack: new_stack, comment }
            }
        }
    }
}
