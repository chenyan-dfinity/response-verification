/*!
# Internet Computer HTTP Certification

HTTP Certification is a sub-protocol of the [Internet Computer](https://internetcomputer.org/) [HTTP Gateway Protocol](https://internetcomputer.org/docs/current/references/http-gateway-protocol-spec). It is used to verify HTTP responses received by an HTTP Gateway from a [canister](https://internetcomputer.org/how-it-works/canister-lifecycle/), with respect to the corresponding HTTP request sent by the HTTP Gateway to the canister. This allows HTTP Gateways to verify that the responses they receive from canisters are authentic and have not been tampered with by a malicious replica.

This crate provides a foundation for implementing the HTTP Certification protocol in Rust canisters. Certification is implemented in a number of steps:

1. [Defining CEL expressions](#defining-cel-expressions)
2. [Creating certifications](#creating-certifications)
3. ...coming soon!!!

## Defining CEL Expressions

[CEL](https://github.com/google/cel-spec) (Common Expression Language) is a portable expression language that can be used to enable different applications to more easily interoperate. It can be seen as the computation or expression counterpart to [Protocol Buffers](https://github.com/protocolbuffers/protobuf).

CEL expressions lie at the heart of the Internet Computer's HTTP certification system. They are used to define the conditions under which a request and response pair should be certified and what should be included from the corresponding request and response objects in the certification.

CEL expressions can be created in two ways, by using the [CEL builder](#using-the-cel-builder), or by directly creating a [CEL expression](#directly-creating-a-cel-expression).

### Converting CEL expressions to their `String` representation

Note that the [CelExpression](cel::CelExpression) enum is not a CEL expression itself, but rather a Rust representation of a CEL expression. To convert a [CelExpression](cel::CelExpression) into its [String] representation, use [CelExpression.to_string](cel::CelExpression::to_string()) or [create_cel_expr](cel::create_cel_expr()). This applies to CEL expressions created both by the [CEL builder](#using-the-cel-builder) and [directly](#directly-creating-a-cel-expression).

```rust
use ic_http_certification::cel::{CelExpression, DefaultCelExpression};

let cel_expr = CelExpression::Default(DefaultCelExpression::Skip).to_string();
```

Alternatively:

```rust
use ic_http_certification::cel::{CelExpression, DefaultCelExpression, create_cel_expr};

let certification = CelExpression::Default(DefaultCelExpression::Skip);
let cel_expr = create_cel_expr(&certification);
```

### Using the CEL builder

The CEL builder interface is provided to ease the creation of CEL expressions through an ergonmic interface. If this interface does not meet your needs, you can also [create CEL expressions directly](#directly-creating-a-cel-expression). To define a CEL expression, start with [DefaultCelBuilder]. This struct provides a set of associated functions that can be used to define how your request and response pair should be certified.

When certifying requests, the request body and method are always certified. To additionally certify request headers and query parameters, use [with_request_headers](cel::DefaultFullCelExpressionBuilder::with_request_headers()) and [with_request_query_parameters](cel::DefaultFullCelExpressionBuilder::with_request_query_parameters()) respectively. Both associated functions take a [str] slice as an argument.

When certifying a response, the response body and status code are always certified. To additionally certify response headers, use [with_response_certification](cel::DefaultFullCelExpressionBuilder::with_response_certification()). This associated function takes the [DefaultResponseCertification](DefaultResponseCertification) enum as an argument. To specify header inclusions, use the [certified_response_headers](DefaultResponseCertification::certified_response_headers) associated function of the [DefaultResponseCertification](DefaultResponseCertification) enum. Or to certify all response headers, with some exclusions, use the [response_header_exclusions](DefaultResponseCertification::response_header_exclusions) associated function of the [DefaultResponseCertification](DefaultResponseCertification) enum. Both associated functions take a [str] slice as an argument.

#### Fully certified request / response pair

To define a fully certified request and response pair, including request headers, query parameters, and response headers use [DefaultCelBuilder::full_certification](DefaultCelBuilder::full_certification()). For example:

```rust
use ic_http_certification::{DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::full_certification()
    .with_request_headers(&["Accept", "Accept-Encoding", "If-None-Match"])
    .with_request_query_parameters(&["foo", "bar", "baz"])
    .with_response_certification(DefaultResponseCertification::certified_response_headers(&[
        "Cache-Control",
        "ETag",
    ]))
    .build();
```

#### Partially certified request

Any number of request headers or request query parameters can be certified via [with_request_headers](cel::DefaultFullCelExpressionBuilder::with_request_headers()) and [with_request_query_parameters](cel::DefaultFullCelExpressionBuilder::with_request_query_parameters()) respectively. Both methods will accept empty arrays, which is the same as not calling them at all. If [with_request_headers](cel::DefaultFullCelExpressionBuilder::with_request_headers()) is called with an empty array, or it is not called at all, then no request headers will be certified. Likewise for [with_request_query_parameters](cel::DefaultFullCelExpressionBuilder::with_request_query_parameters()), if it is called with an empty array, or not called at all, then no request query parameters will be certified. If both are called with an empty array, or neither are called, then only the request body and method will be certified.

For example, to certify only the request body and method:

```rust
use ic_http_certification::{DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::full_certification()
    .with_response_certification(DefaultResponseCertification::certified_response_headers(&[
        "Cache-Control",
        "ETag",
    ]))
    .build();
```

Alternatively, this can be done more explicitly:

```rust
use ic_http_certification::{DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::full_certification()
    .with_request_headers(&[])
    .with_request_query_parameters(&[])
    .with_response_certification(DefaultResponseCertification::certified_response_headers(&[
        "Cache-Control",
        "ETag",
    ]))
    .build();
```

#### Skipping request certification

Request certification can be skipped entirely by using [DefaultCelBuilder::response_only_certification](DefaultCelBuilder::response_only_certification()) instead of [DefaultCelBuilder::full_certification](DefaultCelBuilder::full_certification()). For example:

```rust
use ic_http_certification::{DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::response_only_certification()
    .with_response_certification(DefaultResponseCertification::response_header_exclusions(&[
        "Date",
        "Cookie",
        "Set-Cookie",
    ]))
    .build();
```

#### Partially certified response

Similiarly to request certification, any number of response headers can be provided via the [certified_response_headers](DefaultResponseCertification::certified_response_headers) associated function of the [DefaultResponseCertification](DefaultResponseCertification) enum when calling [with_response_certification](cel::DefaultFullCelExpressionBuilder::with_response_certification()). The provided array can also be an empty. If the array is empty, or the associated function is not called, then no response headers will be certified.

For example, to certify only the response body and status code:

```rust
use ic_http_certification::DefaultCelBuilder;

let cel_expr = DefaultCelBuilder::response_only_certification().build();
```


This can also be done more explicitly:

```rust
use ic_http_certification::{DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::response_only_certification()
    .with_response_certification(DefaultResponseCertification::certified_response_headers(&[]))
    .build();
```

The same applies both when using [DefaultCelBuilder::response_only_certification](DefaultCelBuilder::response_only_certification()) and [DefaultCelBuilder::full_certification](DefaultCelBuilder::full_certification()).

```rust
use ic_http_certification::DefaultCelBuilder;

let cel_expr = DefaultCelBuilder::full_certification()
    .with_request_headers(&["Accept", "Accept-Encoding", "If-None-Match"])
    .with_request_query_parameters(&["foo", "bar", "baz"])
    .build();
```

To skip response certification completely, then certification overall must be skipped completely. It wouldn't be useful to certify a request without certifying a response. So if anything is certified, then it must at least include the response. See the next section for more details on skipping certification entirely.

#### Skipping certification

To skip certification entirely, use [DefaultCelBuilder::skip_certification](DefaultCelBuilder::skip_certification()), for example:

```rust
use ic_http_certification::DefaultCelBuilder;

let cel_expr = DefaultCelBuilder::skip_certification();
```

Skipping certification may seem counter-intuitive at first, but it is not always possible to certify a request and response pair. For example, a canister method that will return different data for every user cannot be easily certified.

Typically these requests have been routed through `raw` Internet Computer URLs in the past, but this is dangerous because `raw` URLs allow any responding replica to decide whether or not certification is required. In contrast, by skipping certification using the above method with a non-`raw` URL, a replica will no longer be able to decide whether or not certification is required and instead this decision will be made by the canister itself and the result will go through consensus.

## Creating certifications

Once a CEL expression has been defined, it can be used in conjunction with an [HTTP request](HttpRequest) and [HTTP response](HttpResponse) to create a [Certification]. The [Certification] enum has three variants, each with a corresponding associated function used to create that particular variant:

- The [Full](Certification::Full) variant is used to include both the [HTTP request](HttpRequest) and the corresponding [HTTP response](HttpResponse) in certification.
- The [ResponseOnly](Certification::ResponseOnly) variant is used to include only the [HTTP response](HttpResponse) in certification and exclude the corresponding [HTTP request](HttpRequest) from certification.
- The [Skip](Certification::Skip) variant is used to skip certification entirely.

### Full certification

To perform a full certification, a CEL expression created from [DefaultCelBuilder::full_certification] is required, along with an [HttpRequest] and [HttpResponse] and optionally, a pre-calculated response body hash. For example:

```rust
use ic_http_certification::{Certification, HttpRequest, HttpResponse, DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::full_certification()
    .with_request_headers(&["Accept", "Accept-Encoding", "If-None-Match"])
    .with_request_query_parameters(&["foo", "bar", "baz"])
    .with_response_certification(DefaultResponseCertification::certified_response_headers(&[
        "Cache-Control",
        "ETag",
    ]))
    .build();

let request = HttpRequest {
    method: "GET".to_string(),
    url: "/index.html?foo=a&bar=b&baz=c".to_string(),
    headers: vec![
        ("Accept".to_string(), "application/json".to_string()),
        ("Accept-Encoding".to_string(), "gzip".to_string()),
        ("If-None-Match".to_string(), "987654321".to_string()),
    ],
    body: vec![],
};

let response = HttpResponse {
    status_code: 200,
    headers: vec![
        ("Cache-Control".to_string(), "no-cache".to_string()),
        ("ETag".to_string(), "123456789".to_string()),
    ],
    body: vec![1, 2, 3, 4, 5, 6],
};

let certification = Certification::full(&cel_expr, &request, &response, None);
```

### Response-only certification

To perform a response-only certification, a CEL expression created from [DefaultCelBuilder::response_only_certification] is required, along with an [HttpResponse] and optionally, a pre-calculated response body hash. For example:

```rust
use ic_http_certification::{Certification, HttpResponse, DefaultCelBuilder, DefaultResponseCertification};

let cel_expr = DefaultCelBuilder::response_only_certification()
    .with_response_certification(DefaultResponseCertification::certified_response_headers(&[
        "Cache-Control",
        "ETag",
    ]))
    .build();

let response = HttpResponse {
    status_code: 200,
    headers: vec![
        ("Cache-Control".to_string(), "no-cache".to_string()),
        ("ETag".to_string(), "123456789".to_string()),
    ],
    body: vec![1, 2, 3, 4, 5, 6],
};

let certification = Certification::response_only(&cel_expr, &response, None);
```

### Skipping certification

Skipping certification does not need an explicit CEL expression to be defined since it's always the same. For example:

```rust
use ic_http_certification::Certification;

let certification = Certification::skip();
```

## Directly creating a CEL expression

To define a CEL expression, start with the [CelExpression](cel::CelExpression) enum. This enum provides a set of variants that can be used to define different types of CEL expressions supported by Internet Computer HTTP Gateways. Currently only one variant is supported, known as the "default" certification expression, but more may be added in the future as the HTTP certification protocol evolves over time.

When certifying requests, the request body and method are always certified. To additionally certify request headers and query parameters, use the [headers](cel::DefaultRequestCertification::headers) and [query_parameters](cel::DefaultRequestCertification::query_parameters) fields of the [DefaultRequestCertification](cel::DefaultRequestCertification) struct. Both fields take a [str] slice as an argument.

When certifying a response, the response body and status code are always certified. To additionally certify response headers, use the [certified_response_headers](DefaultResponseCertification::certified_response_headers) associated function of the [DefaultResponseCertification](DefaultResponseCertification) enum. Or to certify all response headers, with some exclusions, use the [response_header_exclusions](DefaultResponseCertification::response_header_exclusions) associated function of the [DefaultResponseCertification](DefaultResponseCertification) enum. Both associated functions take a [str] slice as an argument.

Note that the example CEL expressions provided below are formatted for readability. The actual CEL expressions produced by [CelExpression::to_string](cel::CelExpression::to_string()) and [create_cel_expr](cel::create_cel_expr()) are minified. The minified CEL expression is preferred because it is more compact, resulting in a smaller payload and a faster evaluation time for the HTTP Gateway that is verifying the certification, but the formatted versions are also accepted.

### Fully certified request / response pair

To define a fully certified request and response pair, including request headers, query parameters, and response headers:

```rust
use std::borrow::Cow;
use ic_http_certification::cel::{CelExpression, DefaultCelExpression, DefaultFullCelExpression, DefaultRequestCertification, DefaultResponseCertification};

let cel_expr = CelExpression::Default(DefaultCelExpression::Full(
  DefaultFullCelExpression {
    request: DefaultRequestCertification {
      headers: Cow::Borrowed(&["Accept", "Accept-Encoding", "If-None-Match"]),
      query_parameters: Cow::Borrowed(&["foo", "bar", "baz"]),
    },
    response: DefaultResponseCertification::certified_response_headers(&[
      "ETag",
      "Cache-Control",
    ]),
  }));
```

This will produce the following CEL expression:

```protobuf
default_certification (
  ValidationArgs {
    request_certification: RequestCertification {
      certified_request_headers: ["Accept", "Accept-Encoding", "If-None-Match"],
      certified_query_parameters: ["foo", "bar", "baz"]
    },
    response_certification: ResponseCertification {
      certified_response_headers: ResponseHeaderList {
        headers: [
          "ETag",
          "Cache-Control"
        ]
      }
    }
  }
)
```

### Partially certified request

Any number of request headers or query parameters can be provided via the [headers](cel::DefaultRequestCertification::headers) and [query_parameters](cel::DefaultRequestCertification::query_parameters) fields of the [DefaultRequestCertification](cel::DefaultRequestCertification) struct, and both can be an empty array. If the [headers](cel::DefaultRequestCertification::headers) field is empty, no request headers will be certified. Likewise for the [query_parameters](cel::DefaultRequestCertification::query_parameters) field, if it is empty then no query parameters will be certified. If both are empty, only the request body and method will be certified.

For example, to certify only the request body and method:

```rust
use std::borrow::Cow;
use ic_http_certification::cel::{CelExpression, DefaultCelExpression, DefaultFullCelExpression, DefaultRequestCertification, DefaultResponseCertification};

let cel_expr = CelExpression::Default(DefaultCelExpression::Full(
  DefaultFullCelExpression {
    request: DefaultRequestCertification {
      headers: Cow::Borrowed(&[]),
      query_parameters: Cow::Borrowed(&[]),
    },
    response: DefaultResponseCertification::certified_response_headers(&[
      "ETag",
      "Cache-Control",
    ]),
  }));
```

This will produce the following CEL expression:

```protobuf
default_certification (
  ValidationArgs {
    request_certification: RequestCertification {
      certified_request_headers: [],
      certified_query_parameters: []
    },
    response_certification: ResponseCertification {
      certified_response_headers: ResponseHeaderList {
        headers: [
          "ETag",
          "Cache-Control"
        ]
      }
    }
  }
)
```

### Skipping request certification

Request certification can be skipped entirely by using the [ResponseOnly](DefaultCelExpression::ResponseOnly) variant of the [DefaultCelExpression](DefaultCelExpression). For example:

```rust
use std::borrow::Cow;
use ic_http_certification::cel::{CelExpression, DefaultCelExpression, DefaultResponseOnlyCelExpression, DefaultResponseCertification};

let cel_expr = CelExpression::Default(DefaultCelExpression::ResponseOnly(
  DefaultResponseOnlyCelExpression {
    response: DefaultResponseCertification::certified_response_headers(&[
      "ETag",
      "Cache-Control",
    ]),
  }));
```

This will produce the following CEL expression:

```protobuf
default_certification (
  ValidationArgs {
    no_request_certification: Empty {},
    response_certification: ResponseCertification {
      certified_response_headers: ResponseHeaderList {
        headers: [
          "ETag",
          "Cache-Control"
        ]
      }
    }
  }
)
```

### Partially certified response

Similiarly to request certification, any number of response headers can be provided via the [certified_response_headers](DefaultResponseCertification::certified_response_headers) associated function of the [DefaultResponseCertification](DefaultResponseCertification) enum, and it can also be an empty array. If the array is empty, no response headers will be certified. For example:

```rust
use std::borrow::Cow;
use ic_http_certification::cel::{CelExpression, DefaultCelExpression, DefaultFullCelExpression, DefaultRequestCertification, DefaultResponseCertification};


let cel_expr = CelExpression::Default(DefaultCelExpression::Full(
  DefaultFullCelExpression {
    request: DefaultRequestCertification {
      headers: Cow::Borrowed(&["Accept", "Accept-Encoding", "If-None-Match"]),
      query_parameters: Cow::Borrowed(&["foo", "bar", "baz"]),
    },
    response: DefaultResponseCertification::certified_response_headers(&[]),
  }));
```

This will produce the following CEL expression:

```protobuf
default_certification (
  ValidationArgs {
    request_certification: RequestCertification {
      certified_request_headers: ["Accept", "Accept-Encoding", "If-None-Match"],
      certified_query_parameters: ["foo", "bar", "baz"]
    },
    response_certification: ResponseCertification {
      certified_response_headers: ResponseHeaderList {
        headers: []
      }
    }
  }
)
```

If the [response_header_exclusions](DefaultResponseCertification::response_header_exclusions) associated function is used, an empty array will certify _all_ response headers. For example:

```rust
use std::borrow::Cow;
use ic_http_certification::cel::{CelExpression, DefaultCelExpression, DefaultFullCelExpression, DefaultRequestCertification, DefaultResponseCertification};

let cel_expr = CelExpression::Default(DefaultCelExpression::Full(
  DefaultFullCelExpression {
    request: DefaultRequestCertification {
      headers: Cow::Borrowed(&["Accept", "Accept-Encoding", "If-None-Match"]),
      query_parameters: Cow::Borrowed(&["foo", "bar", "baz"]),
    },
    response: DefaultResponseCertification::response_header_exclusions(&[]),
  }));
```

This will produce the following CEL expression:

```protobuf
default_certification (
  ValidationArgs {
    request_certification: RequestCertification {
      certified_request_headers: ["Accept", "Accept-Encoding", "If-None-Match"],
      certified_query_parameters: ["foo", "bar", "baz"]
    },
    response_certification: ResponseCertification {
      response_header_exclusions: ResponseHeaderList {
        headers: []
      }
    }
  }
)
```

To skip response certification completely, then certification overall must be skipped completely. It wouldn't be useful to certify a request without certifying a response. So if anything is certified, then it must at least include the response. See the next section for more details on skipping certification entirely.

### Skipping certification

To skip certification entirely:

```rust
use ic_http_certification::cel::{CelExpression, DefaultCelExpression};

let cel_expr = CelExpression::Default(DefaultCelExpression::Skip);
```

This will produce the following CEL expression:

```protobuf
default_certification (
  ValidationArgs {
    no_certification: Empty {}
  }
)
```

Skipping certification may seem counter-intuitive at first, but it is not always possible to certify a request and response pair. For example, a canister method that will return different data for every user cannot be easily certified.

Typically these requests have been routed through `raw` Internet Computer URLs in the past, but this is dangerous because `raw` URLs allow any responding replica to decide whether or not certification is required. In contrast, by skipping certification using the above method with a non-`raw` URL, a replica will no longer be able to decide whether or not certification is required and instead this decision will be made by the canister itself and the result will go through consensus.

*/

#![deny(
    missing_docs,
    missing_debug_implementations,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]

pub mod cel;
pub use cel::{
    CelExpression, DefaultCelBuilder, DefaultCelExpression, DefaultFullCelExpression,
    DefaultResponseCertification, DefaultResponseOnlyCelExpression,
};
pub mod hash;
pub use hash::*;
pub mod error;
pub use error::*;
pub mod http;
pub use crate::http::*;
pub mod tree;
pub use tree::*;
