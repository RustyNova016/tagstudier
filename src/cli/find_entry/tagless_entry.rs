use inquire_derive::Selectable;

#[derive(Debug, Clone, Copy, Selectable, strum::Display)]
enum Response {
    No,
    Yes,
}

pub(super) fn ask_tagless() -> bool {
    let res = Response::select("is this your entry?").prompt().unwrap();

    match res {
        Response::Yes => true,
        Response::No => false,
    }
}
