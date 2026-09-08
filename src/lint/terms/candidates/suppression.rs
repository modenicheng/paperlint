use crate::{lint::terms::AcronymDefinitionEntry, text::lexicon::Lexicon};
use std::collections::HashSet;

pub(super) struct SuppressedSurfaces<'a> {
    declarations: HashSet<String>,
    lexicon: &'a Lexicon,
}

impl<'a> SuppressedSurfaces<'a> {
    pub(super) fn new(lexicon: &'a Lexicon, declarations: &[AcronymDefinitionEntry]) -> Self {
        let mut declared = HashSet::new();
        for declaration in declarations {
            declared.insert(declaration.acronym.clone());
            if let Some(chinese) = &declaration.chinese {
                declared.insert(chinese.clone());
            }
            if let Some(english) = &declaration.english {
                declared.insert(english.clone());
            }
        }
        Self {
            declarations: declared,
            lexicon,
        }
    }

    pub(super) fn contains(&self, surface: &str) -> bool {
        self.declarations.contains(surface) || self.lexicon.lookup(surface).is_some()
    }
}
