use inquire::Select;
use tagstudio_db::Entry;

use crate::ColEyre;
use crate::ColEyreVal;

pub struct EnumQuestion {
    question: String,
    choices: Vec<String>,

    condition: String,
}

impl EnumQuestion {
    pub fn ask(&self, entry: &Entry) -> ColEyre {
        let tag = Select::new(&self.question, self.choices.clone()).prompt()?;

        entry.add_tag_id(conn, tag_id)
    }
}
