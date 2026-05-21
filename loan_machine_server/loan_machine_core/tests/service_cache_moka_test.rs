use loan_machine_models::wallet_address::WalletAddress;
use loan_machine_core::services::subgraph::SubgraphService;
use loan_machine_core::services::cache::MembershipCache;
    use serde_json::json;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Build a wallet with a controllable last byte so tests can spawn
    /// distinct addresses without caring about the exact hex.
    fn wallet(last_byte: u8) -> WalletAddress {
        format!("0x00000000000000000000000000000000000000{:02x}", last_byte)
            .parse()
            .expect("valid wallet")
    }

    /// Stand up a mock subgraph that returns the given `memberRegisteredEvents`
    /// array for every POST. The returned server records each request so the
    /// test can assert how many times the cache actually hit the network.
    async fn mock_subgraph(members: serde_json::Value) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": { "memberRegisteredEvents": members }
            })))
            .mount(&server)
            .await;
        server
    }

    // ── Correctness of the underlying query ──────────────────

    #[tokio::test]
    async fn returns_true_when_member_event_exists() {
        let server   = mock_subgraph(json!([{"id":"evt-1"}])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();

        let got = cache.is_member(&subgraph, &wallet(1), "0xcoop").await.unwrap();
        assert!(got);
    }

    #[tokio::test]
    async fn returns_false_when_no_match() {
        let server   = mock_subgraph(json!([])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();

        let got = cache.is_member(&subgraph, &wallet(1), "0xcoop").await.unwrap();
        assert!(!got);
    }

    // ── Caching behavior ─────────────────────────────────────

    #[tokio::test]
    async fn second_call_with_same_key_is_served_from_cache() {
        let server   = mock_subgraph(json!([{"id":"evt-1"}])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();
        let w        = wallet(1);

        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();
        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();

        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn negative_results_are_cached_too() {
        // If false weren't cached, anyone hitting an unknown coop_id
        // would bypass the cache forever — both a perf and an abuse vector.
        let server   = mock_subgraph(json!([])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();
        let w        = wallet(1);

        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();
        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();

        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn different_wallets_get_separate_entries() {
        let server   = mock_subgraph(json!([{"id":"evt-1"}])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();

        cache.is_member(&subgraph, &wallet(1), "0xcoop").await.unwrap();
        cache.is_member(&subgraph, &wallet(2), "0xcoop").await.unwrap();

        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn different_coops_get_separate_entries() {
        let server   = mock_subgraph(json!([{"id":"evt-1"}])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();
        let w        = wallet(1);

        cache.is_member(&subgraph, &w, "0xcoopA").await.unwrap();
        cache.is_member(&subgraph, &w, "0xcoopB").await.unwrap();

        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn invalidate_forces_a_refetch() {
        let server   = mock_subgraph(json!([{"id":"evt-1"}])).await;
        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();
        let w        = wallet(1);

        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();   // miss
        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();   // hit
        cache.invalidate(&w, "0xcoop").await;
        cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();   // miss again

        assert_eq!(server.received_requests().await.unwrap().len(), 2);
    }

    // ── Error handling ───────────────────────────────────────

    #[tokio::test]
    async fn subgraph_error_is_not_cached() {
        // A transient 500 must NOT poison the cache for the full TTL,
        // or a five-second blip locks users out of their coops for a minute.
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": { "memberRegisteredEvents": [{"id":"evt-1"}] }
            })))
            .mount(&server)
            .await;

        let subgraph = SubgraphService::new(server.uri());
        let cache    = MembershipCache::new();
        let w        = wallet(1);

        assert!(cache.is_member(&subgraph, &w, "0xcoop").await.is_err());
        let ok = cache.is_member(&subgraph, &w, "0xcoop").await.unwrap();
        assert!(ok);
    }
