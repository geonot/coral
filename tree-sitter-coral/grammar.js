module.exports = grammar({
  name: 'coral',

  extras: $ => [/\s/, $.comment],

  rules: {
    program: $ => repeat($._item),

    _item: $ => choice(
      $.statement,
      /\n/,
    ),

    statement: $ => choice(
      $.store_actor,
      $.assignment,
      $.function_def,
      $.object_def,
      $.property,
      $.special_field,
      $.message_handler,
      $.expression,
    ),

    store_actor: $ => seq(
      'store',
      'actor',
      $.identifier,
      optional($.block)
    ),

    assignment: $ => seq(
      $.identifier,
      'is',
      $.expression
    ),

    function_def: $ => seq(
      'fn',
      $.identifier,
      'with',
      $.identifier,
      repeat(seq(',', $.identifier)),
      optional($.block)
    ),

    object_def: $ => seq(
      'object',
      $.identifier,
      optional($.block)
    ),

    property: $ => seq(
      $.identifier,
      optional(seq('?', $.expression))
    ),

    special_field: $ => seq('&', $.identifier),

    message_handler: $ => seq(
      '@',
      $.identifier,
      repeat($.identifier)
    ),

    expression: $ => choice(
      $.identifier,
      $.string,
      $.number,
      $.boolean,
      seq($.identifier, repeat1($.identifier)), // function calls
      seq($.expression, '.', $.identifier),     // method calls
      seq($.expression, 'at', $.expression),    // array access
    ),

    block: $ => seq(
      /\n/,
      /[ \t]+/,
      repeat($._item)
    ),

    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,
    string: $ => choice(
      /"[^"]*"/,
      /'[^']*'/
    ),
    number: $ => /\d+(\.\d+)?/,
    boolean: $ => choice('true', 'false', 'yes', 'no'),
    comment: $ => /#.*/,
  },
})