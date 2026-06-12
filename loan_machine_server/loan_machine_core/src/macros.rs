/// Generates an `async fn` that hides the single-field `Data` wrapper struct.
///
/// Use for queries where the GraphQL response has one root field and the
/// `map` closure is a one-liner. For multi-field responses or complex maps,
/// write the function manually.
///
/// # Example
/// ```ignore
/// gql_fetch! {
///     pub async fn fetch_cooperatives(subgraph: &SubgraphService)
///         -> Result<Vec<CooperativeRow>, SubgraphError>
///     {
///         query: COOPERATIVES_QUERY,
///         vars:  (),
///         data:  { cooperatives: Vec<CooperativeRow> },
///         map:   |d| Ok(d.cooperatives),
///     }
/// }
/// ```

//// macro_rules! work with pattern and basicaly the recived template and the return template.
//// to track the elements it uses metavariable that bind one thing on the received template and then we can use it on the return template.

/* the macro has some elements that are the treated form of some common values, that tell the compiler what to look for, ident (indetificators like name of functions, variables, etc), 
ty (types), expr (expressions), and some other like vis for visibility. They are pre-prepared to be used 
more easely, insted of parsing everything. everything with $label is just a label*/
#[macro_export]
macro_rules! gql_fetch {
    (
        $vis:vis async fn $fn_name:ident(  //// $vis: visibility of the function (pub or not), $fn_name: name of the function. async fn is fixed. 
            subgraph: &SubgraphService //// the first parameter is always the subgraph service reference. Becuse of our purpose with this macro.
            $(, $param:ident : $param_ty:ty)* /* this part is for the other parameters that the function can receive, it is optional and can be
                                             zero or more. It uses repetition with the $(...)* */
        ) -> Result<$out:ty, $err:ty> ////the return part of the function with $out: the type of the success value and $err: the type of the error value.
        {
            /* the macro can have some literal tokens that the matcher requires verbatim. They're not a language feature — they're just identifiers you insist appear at the 
            call site to make it readable. If you wrote them in a different order it wouldn't match.*/

            query: $query:expr, //// the GraphQL query as a string expression.
            vars:  $vars:expr, //// the variables for the GraphQL query, also as an expression. It can be a struct or a tuple, depending on the query needs.
            data:  { $field:ident : $field_ty:ty }, /* this part defines the expected shape of the GraphQL response. It assumes that the response has a single
                                                root field, which is named $field and has type $field_ty. The macro will generate a wrapper struct to deserialize this response.*/
            map:   |$d:ident| $body:expr, /* the mapping closure that takes the deserialized data (wrapped in the struct defined above) and produces the final
                                        result. The $d is the identifier for the deserialized data, and $body is the expression that computes the final result from it.*/
                                        
                                        /* !!!!! more strict with the concepts: `|d:ident| $body:expr` *looks* like a closure but the macro never builds one. It just extracts the name `
                                        d` and the expression `body`, then emits `let d: __Data = ...;` followed by the bare `body`. The closure syntax is cosmetic
                                        — it reads like `.map()` but compiles to a plain `let` + tail expression. Calling it "the mapping closure" is fine as long
                                        as you know no closure exists in the output. */
        }
    ) => { //// here is the output template, where we delegate work to the macro insted of writing it manually every time.
        $vis async fn $fn_name(
            subgraph: &$crate::services::subgraph::SubgraphService,
            $($param: $param_ty),* 
        ) -> Result<$out, $err> {
            #[derive(serde::Deserialize)]
            struct __Data { $field: $field_ty } /* insted of building different structs many times, we can build it from the data
                                                 fields (wich has the struct with the field reuturned by query)*/

            let $d: __Data = subgraph.query($query, $vars).await?; /* and insted of writing the query and the deserialization every time, the macro can do this for us.*/
            $body
        }
    };
}

/// Resolves the on-chain LoanMachine context for a cooperative.
///
/// Almost every `*_logic` function starts the same way: parse the
/// `coop_id_hex` string into a `B256`, look up the deployed LoanMachine
/// address for that coop, and grab the provider. This macro expands to
/// a block expression yielding `(coop_id_b32, lm_addr, provider)` — the
/// caller destructures it with `let`.
///
/// We can't have the macro introduce `let coop_id_b32 = ...` directly
/// into the caller's scope: identifiers created *inside* a `macro_rules!`
/// expansion are hygienically separate from identifiers written at the
/// call site, even if spelled the same. Returning a tuple sidesteps this
/// — the binding names in `let (coop_id_b32, lm_addr, provider) = ...`
/// belong to the caller, so they're ordinary, referenceable locals.
///
/// It uses `?` twice (on the `parse` and on `get_loan_machine`), so the
/// enclosing function must return `Result<_, E>` where `E` has a
/// `#[from] BlockchainError` variant (for the second `?`); the first `?`
/// uses whatever error expression you pass as `invalid_coop_id`.
///
/// # Example
/// ```ignore
/// let (coop_id_b32, lm_addr, provider) = coop_context!(
///     blockchain:      blockchain,
///     coop_id_hex:     coop_id_hex,
///     invalid_coop_id: AdminApprovalError::InvalidCoopId(coop_id_hex.into()),
/// );
/// ```
///
/// Pass the extra `contract` flag to also get a `LoanMachine` instance
/// (built from `lm_addr` and `provider.clone()`) as a fourth element:
///
/// ```ignore
/// let (coop_id_b32, lm_addr, provider, contract) = coop_context!(
///     blockchain:      blockchain,
///     coop_id_hex:     coop_id_hex,
///     invalid_coop_id: AdminApprovalError::InvalidCoopId(coop_id_hex.into()),
///     contract,
/// );
/// ```
#[macro_export]
macro_rules! coop_context {
    (
        blockchain:      $blockchain:expr,
        coop_id_hex:     $coop_id_hex:expr,
        invalid_coop_id: $err:expr $(,)?
    ) => {{
        let coop_id_b32 = $coop_id_hex.parse::<alloy::primitives::B256>()
            .map_err(|_| $err)?;
        let lm_addr  = $blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
        let provider = $blockchain.coop_registry.provider.clone();
        (coop_id_b32, lm_addr, provider)
    }};

    (
        blockchain:      $blockchain:expr,
        coop_id_hex:     $coop_id_hex:expr,
        invalid_coop_id: $err:expr,
        contract $(,)?
    ) => {{
        let coop_id_b32 = $coop_id_hex.parse::<alloy::primitives::B256>()
            .map_err(|_| $err)?;
        let lm_addr  = $blockchain.coop_registry.get_loan_machine(coop_id_b32).await?;
        let provider = $blockchain.coop_registry.provider.clone();
        let contract = $crate::services::blockchain::abis::LoanMachine::new(lm_addr, provider.clone());
        (coop_id_b32, lm_addr, provider, contract)
    }};
}
