use std::fmt;

use super::tag::HtmlTag;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HtmlTagStack(Vec<HtmlTag>);

impl HtmlTagStack {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn from_list(tags: Vec<HtmlTag>) -> Self {
        Self(tags.into_iter().rev().collect())
    }

    pub fn to_list(&self) -> Vec<HtmlTag> {
        let mut out = self.0.clone();
        out.reverse();
        out
    }

    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn last(&self) -> Option<HtmlTag> {
        self.0.first().copied()
    }

    pub fn rebase(&self, base: &HtmlTagStack, new_base: &HtmlTagStack) -> HtmlTagStack {
        if self.0.len() < base.0.len() {
            return self.clone();
        }
        if self.0[self.0.len() - base.0.len()..] == base.0[..] {
            let mut rebased = self.0[..self.0.len() - base.0.len()].to_vec();
            rebased.extend(new_base.0.iter().copied());
            HtmlTagStack(rebased)
        } else {
            self.clone()
        }
    }

    pub fn push(&self, tag: HtmlTag) -> HtmlTagStack {
        let mut tags = self.0.clone();
        tags.insert(0, tag);
        HtmlTagStack(tags)
    }

    pub fn pop(&self, tag: HtmlTag) -> HtmlTagStack {
        if self.0.is_empty() {
            return HtmlTagStack::empty();
        }
        if self.0[0] == tag {
            return HtmlTagStack(self.0[1..].to_vec());
        }

        let split = self.0.iter().position(|&t| t == tag);
        match split {
            Some(idx) => {
                let mut tags = self.0[..idx].to_vec();
                tags.extend_from_slice(&self.0[idx + 1..]);
                HtmlTagStack(tags)
            }
            None => self.clone(),
        }
    }

    pub fn descends_from(&self, other: &HtmlTagStack) -> bool {
        if other.0.len() > self.0.len() {
            return false;
        }
        self.0[self.0.len() - other.0.len()..] == other.0[..]
    }

    pub fn any<F>(&self, mut f: F) -> bool
    where
        F: FnMut(HtmlTag) -> bool,
    {
        self.0.iter().copied().any(&mut f)
    }

    pub fn contains(&self, tag: HtmlTag) -> bool {
        self.0.contains(&tag)
    }
}

impl<const N: usize> From<[HtmlTag; N]> for HtmlTagStack {
    fn from(value: [HtmlTag; N]) -> Self {
        HtmlTagStack::from_list(value.to_vec())
    }
}

impl From<Vec<HtmlTag>> for HtmlTagStack {
    fn from(value: Vec<HtmlTag>) -> Self {
        HtmlTagStack::from_list(value)
    }
}

impl fmt::Debug for HtmlTagStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("fromList ")?;
        f.debug_list().entries(self.to_list()).finish()
    }
}
