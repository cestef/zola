/// MathML will italic some single characters that typst does not italic
/// by default, therefore we need to add `mathvariant="normal"` manually.
pub fn is_normal(c: char) -> bool {
    !matches!(
        c,
        'a'..='z' | 'ħ' | 'ı' | 'ȷ' | 'A'..='Z' |
        'α'..='ω' | '∂' | 'ϵ' | 'ϑ' | 'ϰ' | 'ϕ' | 'ϱ' | 'ϖ'
    )
}
