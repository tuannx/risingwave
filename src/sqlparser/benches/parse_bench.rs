// Copyright 2026 RisingWave Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Perf baseline bench for the `risingwave_sqlparser` leaf crate.
//!
//! Added for the fork perf-test rollout (see `perf/` at repo root).
//! The first two cases reuse the fixed SQL strings from the orphaned
//! `sqlparser_bench` crate (`src/sqlparser/sqlparser_bench`), which is not
//! a workspace member and therefore cannot be built as-is; they are adapted
//! to this crate's dialect-free `Parser::parse_sql(sql)` API. The third
//! case covers a streaming DDL statement typical for RisingWave.
//! Inputs are fixed constants so reruns are deterministic.

use std::time::Duration;

use criterion::{criterion_group, criterion_main, Criterion};
use risingwave_sqlparser::parser::Parser;

const SELECT_QUERY: &str = "SELECT * FROM table WHERE 1 = 1";

const WITH_QUERY: &str = "
        WITH derived AS (
            SELECT MAX(a) AS max_a,
                   COUNT(b) AS b_num,
                   user_id
            FROM TABLE
            GROUP BY user_id
        )
        SELECT * FROM table
        LEFT JOIN derived USING (user_id)
    ";

const CREATE_MV_QUERY: &str = "CREATE MATERIALIZED VIEW mv1 AS SELECT user_id, COUNT(*) AS cnt, \
     SUM(amount) AS total FROM orders WHERE amount > 100 GROUP BY user_id";

fn parse_benches(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqlparser_parse");
    // Fixed configuration for reproducible reruns on a shared VM:
    // sample_size=50, warmup=2s, measurement=8s per bench.
    group
        .sample_size(50)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(8));

    group.bench_function("parse_select", |b| {
        b.iter(|| Parser::parse_sql(SELECT_QUERY));
    });
    group.bench_function("parse_with_select", |b| {
        b.iter(|| Parser::parse_sql(WITH_QUERY));
    });
    group.bench_function("parse_create_mv", |b| {
        b.iter(|| Parser::parse_sql(CREATE_MV_QUERY));
    });
    group.finish();
}

criterion_group!(benches, parse_benches);
criterion_main!(benches);
