; Keywords
[
  "fn"
  "object"
  "store"
  "if"
  "else"
  "for"
  "in"
  "while"
  "return"
  "with"
  "extends"
  "is"
  "log"
] @keyword

; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "gt"
  "lt"
  "equals"
  "gte"
  "lte"
  "and"
  "or"
  "at"
  "?"
  "!"
] @operator

; Literals
(string_literal) @string
(interpolated_string) @string
(number_literal) @number
(boolean_literal) @boolean

; Identifiers
(identifier) @variable

; Function names
(function_definition name: (identifier) @function)
(call_expression function: (identifier) @function.call)

; Types/Classes
(object_definition name: (identifier) @type)
(store_definition name: (identifier) @type)

; Properties
(member_expression property: (identifier) @property)

; Comments
(comment) @comment

; Punctuation
[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
  "."
  ","
] @punctuation.delimiter

; String interpolation
(interpolation 
  "{" @punctuation.special
  "}" @punctuation.special)

(interpolation (_) @embedded)