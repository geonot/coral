// Using `tree-sitter-cli` version 0.20.x for `grammar` and `PREC`
// For indentation, an external scanner would be ideal.
// This grammar will assume newlines are significant for block termination
// and use `optional($._indented_block)` which would rely on such a scanner
// or a very careful rule structure. For now, `$.block` is a simplified placeholder.

const PREC = {
  assign: -1,
  ternary: 1,
  logical_or: 2,
  logical_and: 3,
  equality: 4,
  comparison: 5,
  additive: 6,
  multiplicative: 7,
  power: 8,
  unary: 9,
  postfix: 10,
  call: 11, // Covers method calls, function calls, array access
};

module.exports = grammar({
  name: 'coral',

  extras: $ => [
    $.comment,
    /\s/, // Includes newlines if not handled by an external scanner explicitly for blocks
  ],

  // Tree-sitter rule names usually use underscores. EBNF names are kept where sensible.
  // `_` prefixed rules are typically hidden from the CST by default.
  rules: {
    program: $ => repeat($._statement_or_newline),

    _statement_or_newline: $ => choice(
      $._statement,
      $._newline // Explicit newline token if needed by grammar structure
    ),

    _statement: $ => choice(
      $.assignment,
      $.function_definition,
      $.object_definition,
      $.store_definition,
      $.actor_definition,
      // $.method_call_statement, // Covered by expression_statement
      // $.function_call_statement, // Covered by expression_statement
      $.control_flow_statement,
      // $.error_handling_statement, // TODO
      $.use_statement,
      $.expression_statement,
    ),

    // Keywords (example, more would be needed)
    kw_fn: $ => 'fn',
    kw_is: $ => 'is',
    kw_with: $ => 'with',
    kw_object: $ => 'object',
    kw_store: $ => 'store',
    kw_actor: $ => 'actor',
    kw_make: $ => 'make',
    kw_push: $ => 'push',
    kw_on: $ => 'on',
    kw_if: $ => 'if',
    kw_else: $ => 'else',
    kw_unless: $ => 'unless',
    kw_while: $ => 'while',
    kw_until: $ => 'until',
    kw_across: $ => 'across',
    kw_iterate: $ => 'iterate',
    kw_use: $ => 'use',
    kw_return: $ => 'return',
    kw_at: $ => 'at',
    kw_as: $ => 'as',
    kw_then: $ => 'then',
    kw_and: $ => 'and', // For logical operator, distinct from '&&' if needed
    kw_or: $ => 'or',   // For logical operator, distinct from '||' if needed
    kw_gt: $ => 'gt',
    kw_lt: $ => 'lt',
    kw_equals: $ => 'equals', // For comparison, distinct from '==' if needed
    kw_gte: $ => 'gte',
    kw_lte: $ => 'lte',
    kw_empty: $ => 'empty',
    kw_now: $ => 'now',
    kw_err: $ => 'err',
    kw_from: $ => 'from',
    kw_by: $ => 'by',
    kw_into: $ => 'into',
    // TODO: Add all other keywords from EBNF's reserved list (e.g. for error_action)

    // From EBNF: assignment = identifier "is" expression ;
    assignment: $ => prec.left(PREC.assign, seq(
      $.identifier,
      $.kw_is,
      $._expression
    )),

    // From EBNF: function_definition = "fn" identifier "with" parameter_list [ indented_block ] ;
    function_definition: $ => choice(
      prec(1, seq($.kw_fn, $.identifier, $.kw_with, optional($.parameter_list), $._indented_block)),
      seq($.kw_fn, $.identifier, $.kw_with, optional($.parameter_list))
    ),

    // From EBNF: parameter_list = [ parameter { "," parameter } ] ;
    // This rule now matches one or more parameters.
    parameter_list: $ => seq(
      $.parameter,
      repeat(seq(',', $.parameter))
    ),

    // From EBNF: parameter = identifier [ default_value ] ;
    // default_value = literal ;
    parameter: $ => seq(
      $.identifier,
      optional(seq('?', $._literal)) // EBNF says `default_value = literal` but often `? expression` is used
    ),

    // From EBNF: object_definition = "object" identifier [ indented_block ] ;
    object_definition: $ => choice(
      prec(1, seq($.kw_object, $.identifier, $._indented_block)),
      seq($.kw_object, $.identifier)
    ),

    // From EBNF: store_definition = "store" identifier [ indented_block ] ;
    store_definition: $ => choice(
      prec(1, seq($.kw_store, $.identifier, $._indented_block)),
      seq($.kw_store, $.identifier)
    ),

    // From EBNF: actor_definition = "store" "actor" identifier [ indented_block ] ;
    actor_definition: $ => choice(
      prec(1, seq($.kw_store, $.kw_actor, $.identifier, $._indented_block)),
      seq($.kw_store, $.kw_actor, $.identifier)
    ),

    // From EBNF: indented_block = newline indent body_content dedent ;
    // This is a simplified block structure. Proper indentation handling needs an external scanner.
    // For now, assuming body_content is a sequence of statements.
    _indented_block: $ => seq(
        $._newline, // Requires explicit newline handling
        $._indent,  // Requires external scanner
        repeat($._body_content_item),
        $._dedent   // Requires external scanner
    ),
    // Placeholder if no external scanner for indent/dedent:
    // block: $ => seq($._newline, repeat1(seq(/[ \t]+/, $._statement, $._newline))),


    // From EBNF: body_content = { property_definition | method_definition | join_table_reference | message_handler | as_method_definition | make_method } ;
    // This will be used inside object_definition, store_definition, actor_definition
    _body_content_item: $ => choice(
      $.property_definition,
      $.method_definition,
      $.join_table_reference,
      $.message_handler,
      $.as_method_definition,
      $.make_method
      // Ensure these are followed by a newline or are the last item before dedent
    ),

    as_method_definition: $ => choice(
      prec(1, seq(
        choice(
            seq($.kw_as, field('type', $.identifier)),
            field('type_prefixed', alias(seq(token('as_'), $.identifier), $.as_prefixed_type))
        ),
        $._indented_block
      )),
      seq(
        choice(
            seq($.kw_as, field('type', $.identifier)),
            field('type_prefixed', alias(seq(token('as_'), $.identifier), $.as_prefixed_type))
        )
      )
    ),

    join_table_reference: $ => seq('&', $.identifier, $._newline), // from EBNF

    message_handler: $ => choice(
      prec(1, seq('@', $.identifier, optional($.parameter_list), $._indented_block)),
      seq('@', $.identifier, optional($.parameter_list))
    ),

    // From EBNF: property_definition = identifier [ "?" default_value ] [ doc_comment ] newline ;
    property_definition: $ => seq(
      $.identifier,
      optional(seq('?', $._expression)), // EBNF default_value is literal, but expression is more common
      // doc_comment is handled by general comment extra
      $._newline // Properties are typically one per line
    ),

    // From EBNF: method_definition = identifier [ parameter_list ] [ indented_block ] ;
    method_definition: $ => choice(
      prec(1, seq($.identifier, optional($.parameter_list), $._indented_block)),
      seq($.identifier, optional($.parameter_list))
    ),

    // From EBNF: make_method = "make" [ indented_block ] ;
    make_method: $ => choice(
      prec(1, seq($.kw_make, $._indented_block)),
      $.kw_make
    ),

    // Control Flow (incomplete, just a few examples)
    control_flow_statement: $ => choice(
      $.if_statement,
      $.push_statement,
      $.while_loop,
      $.unless_statement,
      $.until_loop,
      $.iteration_statement
    ),

    if_statement: $ => choice(
        prec.right(1, seq( // Dangling else resolved by giving if-else higher precedence/right assoc
            $.kw_if,
            field('condition', $._expression),
            field('then_branch', $._statement_block),
            $.kw_else,
            field('else_branch', $._statement_block)
        )),
        seq( // if without else
            $.kw_if,
            field('condition', $._expression),
            field('then_branch', $._statement_block)
        )
    ),

    while_loop: $ => seq(
      $.kw_while,
      field('condition', $._expression),
      field('body', $._statement_block)
    ),

    until_loop: $ => seq( // EBNF: "until" identifier [ "from" expression ] [ "by" expression ] "is" expression statement_block
        $.kw_until,
        field('iterator', $.identifier),
        optional(seq($.kw_from, field('start_value', $._expression))),
        optional(seq($.kw_by, field('step_value', $._expression))),
        $.kw_is, // 'is' is the separator for the end condition
        field('end_condition', $._expression),
        field('body', $._statement_block)
    ),

    iteration_statement: $ => choice( // EBNF: iteration = across_iteration | iterate_statement ;
        $.across_iteration,
        $.iterate_statement
    ),

    across_iteration: $ => seq( // EBNF: expression "across" expression [ "with" named_argument_list ] [ "into" identifier ] ;
        field('target_or_function', $._expression), // This could be function name if first expression is just identifier
        $.kw_across,
        field('collection', $._expression),
        optional(seq($.kw_with, field('named_args', $.named_argument_list))),
        optional(seq($.kw_into, field('result_var', $.identifier)))
    ),

    iterate_statement: $ => choice(
        prec(1, seq( // Prefer with parameter_reference
            $.kw_iterate,
            field('collection', $._expression),
            field('function_or_block', $._expression),
            field('param_ref', $.parameter_reference)
        )),
        seq( // Fallback if no parameter_reference
            $.kw_iterate,
            field('collection', $._expression),
            field('function_or_block', $._expression)
        )
    ),

    unless_statement: $ => choice( // EBNF: "unless" expression statement_block | statement "unless" expression ;
        seq($.kw_unless, field('condition', $._expression), field('body', $._statement_block)),
        // TODO: Postfix unless: seq($._statement, $.kw_unless, field('condition', $._expression))
        // This requires careful thought on how statements and expressions are structured to avoid conflicts.
    ),

    push_statement: $ => seq(
        $.kw_push,
        $._expression,
        $.kw_on,
        $._expression
    ),

    _statement_block: $ => choice(
        $._indented_block,
        $._statement // single statement
    ),

    use_statement: $ => seq(
        $.kw_use,
        $.module_list
    ),

    module_list: $ => seq($.module_name, repeat(seq(',', $.module_name))),
    module_name: $ => prec.left(seq($.identifier, repeat(seq('.', $.identifier)))),


    // Expression Statements
    expression_statement: $ => $._expression,

    // Expression hierarchy based on EBNF precedence
    _expression: $ => $.ternary_expression,

    ternary_expression: $ => prec.right(PREC.ternary, seq(
      $.logical_or_expression,
      optional(seq('?', $._expression, '!', $._expression))
    )),

    logical_or_expression: $ => prec.left(PREC.logical_or, seq(
      $.logical_and_expression,
      repeat(seq(choice('or', '||'), $.logical_and_expression))
    )),

    logical_and_expression: $ => prec.left(PREC.logical_and, seq(
      $.equality_expression,
      repeat(seq(choice('and', '&&'), $.equality_expression))
    )),

    equality_expression: $ => prec.left(PREC.equality, seq(
      $.comparison_expression,
      repeat(seq(choice('==', '!=', 'equals'), $.comparison_expression))
    )),

    comparison_expression: $ => prec.left(PREC.comparison, seq(
      $.additive_expression,
      repeat(seq(choice('gt', 'lt', 'gte', 'lte', '>=', '<='), $.additive_expression))
    )),

    additive_expression: $ => prec.left(PREC.additive, seq(
      $.multiplicative_expression,
      repeat(seq(choice('+', '-'), $.multiplicative_expression))
    )),

    multiplicative_expression: $ => prec.left(PREC.multiplicative, seq(
      $.power_expression,
      repeat(seq(choice('*', '/', '%'), $.power_expression))
    )),

    power_expression: $ => prec.right(PREC.power, seq(
      $._unary_base_for_power, // This will be the new unary_expression rule
      optional(seq('**', $.power_expression))
    )),

    // unary_expression from EBNF: ( "!" | "-" | "~" ) unary_expression | postfix_expression ;
    // This means it can be a prefixed version of itself, or fall back to postfix.
    // Let's call what power_expression uses `_unary_base_for_power` to avoid direct left recursion issues in some structures.
    // And define unary_expression to include both prefixed and non-prefixed (postfix) forms.
    _unary_base_for_power: $ => choice( // This is what higher precedence ops (like power) operate on.
        prec.right(PREC.unary, seq(choice('!', '-', '~'), $._unary_base_for_power)),
        $.postfix_expression
    ),
    // This `unary_expression` is the general one that fits into the expression chain.
    // It seems `_unary_base_for_power` is actually the correct `unary_expression` for the EBNF.
    // Let's rename it to `unary_expression` and ensure the chain is correct.
    // multiplicative_expression -> power_expression -> unary_expression -> postfix_expression -> primary_expression

    // Corrected unary_expression based on EBNF:
    unary_expression: $ => choice(
      prec.right(PREC.unary, seq(choice('!', '-', '~'), $.unary_expression)), // Prefix form
      $.postfix_expression                                                 // Non-prefix form (base)
    ),

    postfix_expression: $ => prec.left(PREC.postfix, seq(
      $.primary_expression,
      repeat($.postfix_operator)
    )),

    primary_expression: $ => choice(
      $._literal,
      $.identifier,
      $.array_literal,
      $.parenthesized_expression,
      $.function_call,
      $.instantiation,
      $.object_literal,
      $.parameter_reference, // Added
      $.built_in_operation   // Added
    ),

    parenthesized_expression: $ => seq('(', $._expression, ')'),

    parameter_reference: $ => seq('$', choice($.identifier, /\d+/, '$')), // EBNF: "$" ( identifier | integer | "$" )

    built_in_operation: $ => choice(
        $.log_operation,
        $.hash_operation,
        $.load_operation,
        $.create_operation,
        $.update_operation
    ),

    log_operation: $ => seq(token('log'), $._expression),
    hash_operation: $ => seq(token('hash'), '.', field('algorithm', $.identifier), $._expression),
    load_operation: $ => seq(token('load'), $._expression),
    create_operation: $ => choice(
        prec(1, seq(token('create'), $.expression_list)),
        token('create')
    ),
    update_operation: $ => choice(
        prec(1, seq(token('update'), field('target', $._expression), $.expression_list)),
        seq(token('update'), field('target', $._expression))
    ),
    expression_list: $ => prec.left(repeat1($._expression)), // Added prec.left

    _literal: $ => choice(
      $.string_literal,
      $.number_literal,
      $.boolean_literal,
      $.empty_literal, // Added
      $.now_literal    // Added
    ),

    string_literal: $ => choice(
      seq("'", repeat(choice(token.immediate(/[^'{}\\]+/), /\\./, $.string_interpolation)), "'"),
      seq('"', repeat(choice(token.immediate(/[^"{}\\]+/), /\\./, $.string_interpolation)), '"')
    ),
    string_interpolation: $ => seq('{', $._expression, '}'),

    number_literal: $ => /\d+(\.\d+)?/,
    boolean_literal: $ => choice('true', 'false', 'yes', 'no'),
    empty_literal: $ => $.kw_empty, // Changed: only the keyword 'empty'. `[]` is handled by array_literal.
    now_literal: $ => $.kw_now,

    array_literal: $ => seq(
        '[',
        optional(seq($._expression, repeat(seq(',', $._expression)))),
        ']'
    ),

    function_call: $ => prec(PREC.call, seq(
        field('name', $.identifier),
        field('arguments', $.argument_list)
    )),

    argument_list: $ => choice(
        seq('(', optional(seq($._expression, repeat(seq(',', $._expression)))), ')'),
        // Removing repeat1($.primary_expression) for space-separated calls for now to reduce ambiguity.
        // If space-separated calls are desired, they need careful handling.
        // E.g., only allow simple identifiers: repeat1($.identifier)
    ),

    // This rule was defined earlier, this is a refinement based on primary_expression
    // postfix_expression rule is already correctly defined above.

    postfix_operator: $ => choice(
        $.method_call_operator,
        $.array_access_bracket_operator,
        $.array_access_at_operator,
        $.as_conversion_operator,
        $.error_handling_operator // Added
    ),

    error_handling_operator: $ => prec.left(PREC.postfix, seq( // Typically error handling is a low precedence postfix op or special statement form
        $.kw_err,
        $.error_action
    )),

    error_action: $ => choice( // EBNF: "log" "return" | default_value | "return" "log" "err" ; default_value is literal
        seq(token('log'), $.kw_return), // Assuming 'log' and 'return' can be keywords or identifiers based on context
        $._literal, // default_value
        seq($.kw_return, token('log'), $.kw_err)
    ),

    method_call_operator: $ => choice(
        prec(PREC.call + 1, seq( // Prefer with method_chaining
            '.',
            field('method', $.identifier),
            optional('!'),
            optional($.argument_list),
            $.method_chaining
        )),
        prec(PREC.call, seq( // Fallback if no method_chaining
            '.',
            field('method', $.identifier),
            optional('!'),
            optional($.argument_list)
        ))
    ),

    method_chaining: $ => prec.right(PREC.call, seq( // right associative for chaining
        choice($.kw_then, $.kw_and),
        '.',
        field('method', $.identifier),
        optional('!'),
        optional($.argument_list),
        optional($.method_chaining) // Recursive chaining
    )),

    array_access_bracket_operator: $ => prec(PREC.call, seq(
        '[', $._expression, ']'
    )),
    array_access_at_operator: $ => prec(PREC.call, seq(
        choice($.kw_at, '@'), $._expression // '@' needs to be defined if not a default token
    )),

    as_conversion_operator: $ => prec(PREC.postfix, seq( // 'as' has similar precedence to other postfix ops
        $.kw_as,
        $.identifier // conversion_type = identifier (or specific like "string", "map", "list")
                     // For simplicity, using identifier. Specific types can be aliased or checked in code.
    )),

    // EBNF: instantiation = identifier [ "!" ] [ argument_list ] | identifier "with" named_argument_list ;
    instantiation: $ => choice(
        prec(PREC.call + 1, seq( // Higher precedence for the version with argument_list after '!'
            $.identifier,
            token.immediate('!'),
            $.argument_list
        )),
        prec(PREC.call, seq( // identifier ! (no arg list)
            $.identifier,
            token.immediate('!')
        )),
        prec(PREC.call, seq( // identifier with named_argument_list
            $.identifier,
            $.kw_with,
            $.named_argument_list
        ))
    ),

    named_argument_list: $ => prec.left(seq(
        $.named_argument,
        repeat(seq(',', $.named_argument))
    )),
    named_argument: $ => seq($.identifier, $._expression), // EBNF: identifier expression

    // EBNF: object_literal = property_assignment { property_assignment } ;
    // EBNF: property_assignment = identifier "is" expression ;
    // Simplified: { key is value, ... }
    object_literal: $ => seq(
        '{', // Assuming Coral object literals use curly braces
        optional(seq($.property_assignment, repeat(seq(',', $.property_assignment)))),
        '}'
    ),
    property_assignment: $ => seq($.identifier, $.kw_is, $._expression),


    identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

    comment: $ => token(seq('#', /.*/)), // Allow single and double hash comments for simplicity here

    // Assuming an external scanner provides these based on indentation
    _newline: $ => token(choice('\n', '\r', '\r\n')), // Basic newline
    _indent: $ => token('indent'), // Placeholder for external scanner token
    _dedent: $ => token('dedent'), // Placeholder for external scanner token

    // `property` and `special_field` from old grammar.js were more like `property_definition`
    // and `join_table_reference` which belong inside body content.
    // `message_handler` also belongs in body content (typically actor).
    // Old `store_actor` is now `actor_definition`.
  },
});