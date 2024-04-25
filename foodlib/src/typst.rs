use typst::{eval::Tracer, foundations::Smart};
use typst_as_library::TypstWrapperWorld;

fn create_pdf(text: String) -> Vec<u8> {
    let world = TypstWrapperWorld::new("./".to_owned(), text);

    let mut tracer = Tracer::default();
    let document = typst::compile(&world, &mut tracer).expect("Error compiling typst");
    let pdf = typst_pdf::pdf(&document, Smart::Auto, None);
    pdf
}
