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
#[macro_export]
macro_rules! gql_fetch {
    (
        $vis:vis async fn $fn_name:ident(
            subgraph: &SubgraphService
            $(, $param:ident : $param_ty:ty)*
        ) -> Result<$out:ty, $err:ty>
        {
            query: $query:expr,
            vars:  $vars:expr,
            data:  { $field:ident : $field_ty:ty },
            map:   |$d:ident| $body:expr,
        }
    ) => {
        $vis async fn $fn_name(
            subgraph: &$crate::services::subgraph::SubgraphService,
            $($param: $param_ty),*
        ) -> Result<$out, $err> {
            #[derive(serde::Deserialize)]
            struct __Data { $field: $field_ty }

            let $d: __Data = subgraph.query($query, $vars).await?;
            $body
        }
    };
}
