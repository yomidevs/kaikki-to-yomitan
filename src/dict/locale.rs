use crate::lang::Lang;

// This should be done differently, and support every section of the dictionary (i.e. Etymology)

// In practice, this is only called for target: Edition, in the main dictionary
pub fn localize_examples_string(target: Lang, n: usize) -> String {
    let (singular, plural) = match target {
        Lang::Fr => ("exemple", "exemples"),
        Lang::De => ("Beispiel", "Beispiele"),
        Lang::Es => ("ejemplo", "ejemplos"),
        Lang::Ru => ("пример", "примеры"),
        Lang::Cs => ("příklad", "příklady"),
        Lang::Nl => ("voorbeeld", "voorbeelden"),
        Lang::El => ("παράδειγμα", "παραδείγματα"),
        Lang::It => ("esempio", "esempi"),
        Lang::Ku => ("nimûne", "nimûneyên"),
        Lang::Pl => ("przykład", "przykłady"),
        Lang::Pt => ("exemplo", "exemplos"),

        // no plural distinction
        Lang::Id | Lang::Ms => ("contoh", "contoh"),
        Lang::Tr => ("örnek", "örnek"),
        Lang::Zh | Lang::Ja => ("例", "例"),
        Lang::Ko => ("예문", "예문"),
        Lang::Th => ("ตัวอย่าง", "ตัวอย่าง"),
        Lang::Vi => ("ví dụ", "ví dụ"),

        _ => ("example", "examples"),
    };

    if n == 1 {
        format!("1 {singular}")
    } else {
        format!("{n} {plural}")
    }
}
