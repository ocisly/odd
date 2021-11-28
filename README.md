# odd ♠️

a Texas Hold'em poker odds simulator available as a [CLI](./cli),
[REST API](./web) and [Rust crate](./engine).

> _he deals the cards to find the answer_  
> _the sacred geometry of chance_  
> _the hidden law of a probable outcome_  
> _the numbers lead a dance..._ 🎶  
> — Sting (Shape of My Heart)

## Rationale

Hold'em is a game of chance and skill. **odd** allows players to explore how
any given situation encountered at the poker table might play out. It does this
by sampling from the space of possible deck shuffles in order to estimate the
odds of winning for each player. By taking care of these mundane calculations,
**odd** allows us humans to focus on perfecting our game.
