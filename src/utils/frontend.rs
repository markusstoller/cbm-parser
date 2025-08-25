/*!
This Rust module is a simple HTTP API server and frontend for a database-backed file search system.
It is built using `hyper` for HTTP handling, `rusqlite` for database interaction,
and `tokio` as the asynchronous runtime. Below is a detailed explanation of different parts of the code:

---

### Imports:
- **`hyper`**: Provides core HTTP server and client functionality.
- **`rusqlite`**: SQLite database interaction library to perform database operations.
- **`tokio`**: Asynchronous runtime to handle I/O-bound operations.
- **`url::form_urlencoded`**: For parsing URL query parameters.
- **`bytes` & `http_body_util`**: Utilities for handling HTTP request/response bodies.

---

### Structs
1. **`FrontendState`**
   - Holds shared application state.
   - Contains `db_path` (an `Arc`-wrapped `String`), representing the database file path.
   - Implemented as `Clone` to ensure safe thread-safe sharing across tasks.

2. **`QueryParams`**
   - Represents parsed query parameters.
   - Fields:
     - `q`: Optional search string.
     - `page`: Current page index (default is 1).
   - Uses `Default` trait for easy initialization.

---

### Functions
#### **Utility/Helper Functions**
1. **`parse_params`**
   - Parses query parameters (`q` and `page`) from an incoming HTTP request.
   - Ensures `page` defaults to `1` and validates input.

2. **`html_header`**
   - Generates the common HTML header, including styles and navigation elements for the frontend pages.

3. **`opt_str`**
   - Short helper function to dereference an `Option<String>` into a `&str`.

4. **`render_form`**
   - Renders a simple HTML form for search input.
   - Encodes text to prevent XSS vulnerabilities.

5. **`encode_query`**
   - Encodes query parameters into a URL-safe string for building links.

6. **`build_sql`**
   - Constructs an SQL query based on user query parameters for searching the `files` table.
   - Handles input sanitization and prepares parameterized statement placeholders (to mitigate SQL injection).

---

#### **Route Handlers**
1. **`handle_index`**
   - Handles requests to the root path `/`.
   - Issues an HTTP 302 (Found) redirect to `/search`, showing the unfiltered list of file entries.

2. **`handle_search`**
   - Core search-handler route (under `/search`).
   - Parses query parameters from the request.
   - Opens a SQLite database using the provided connection path.
   - Runs a paginated SQL query based on user input (`q` and `page`) to fetch matching file records.
   - Generates an HTML response that includes:
     - Search form field.
     - Summary of results (total matches and pagination details).
     - A table listing file records with attributes such as filename, size, checksum hashes, etc.

---

### Constant/Configurable Values:
1. **Page Size**:
   - Defined as `100`. Controls the number of results shown per page.

---

### Detailed Error Handling:
- Error handling is implemented throughout the code, leveraging `Result` and panic-safe patterns.
- For database-related issues (connection or query preparation), meaningful `500 Internal Server Error` HTML responses are returned with detailed error messages.
- Invalid parameters are sanitized or assigned default values.

---

### Example Usage Workflow:
1. User accesses `/` path:
   - Redirects to `/search`.
2. User interacts with the search form and submits a query parameter (`q`).
   - Backend executes a filtered paginated SQL query.
   - Results are rendered as an HTML table with navigation and search.

---

### Dependencies:
1. **`hyper`**: Async HTTP server and connection handling.
2. **`rusqlite`**: SQLite adapter for database storage.
3. **`tokio`**: Async runtime support for I/O processing.
4. **`url`**: Query parameter parsing and encoding.
5. **`html_escape`**: Used to escape text for HTML safety to prevent injection attacks.

---

### Extensibility Ideas:
- Add authentication or authorization layer for securing the application.
- Enable more advanced filtering parameters (e.g., date ranges or file size filtering).
- Serve static files or assets (e.g., images, styles).
- Support additional APIs for JSON-based query results.

---

### Notes on Concurrency:
1. `Arc` (Atomic Reference Counting) is used to safely share immutable state across asynchronous tasks.
2. SQLite itself works well in a single-threaded environment by default. For scalability, using a connection pool (e.g., via `r2d2`) may be preferred.
*/
use hyper::{header, server::conn::http1, Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::Full;
use bytes::Bytes;
use rusqlite::{Connection, ToSql};
use std::{convert::Infallible, net::SocketAddr, sync::Arc, path::Path};
use tokio::net::TcpListener;
use url::form_urlencoded;
use hyper::service::Service;

#[derive(Clone)]
struct FrontendState {
    db_path: Arc<String>,
}

#[derive(Default, Debug)]
struct QueryParams {
    q: Option<String>,
    page: usize,
}

/// Parses query parameters from the given HTTP request and returns them as a `QueryParams` object.
///
/// This function extracts query parameters from the URL portion of the incoming HTTP request
/// and maps them into a `QueryParams` structure. If a parameter is not present or cannot
/// be correctly parsed, default values are used.
///
/// # Parameters
/// - `req`: A reference to the incoming HTTP `Request` object containing the URI and query string.
///
/// # Returns
/// - A `QueryParams` object populated with the parsed parameters or default values.
///
/// ## Behavior
/// - The `q` parameter is captured as a `String` and assigned to the `q` field of the `QueryParams` struct.
/// - The `page` parameter is parsed as a `usize`:
///   - If parsing is successful, the value is assigned to the `page` field.
///   - If parsing fails or the value is less than 1, a default value of 1 is used.
/// - If other query parameters are present, they are ignored.
///
/// ## Default Values
/// - If no `page` parameter is present, the `page` is set to 1.
/// - If no parameters are present, the `q` field remains `None`.
///
/// ## Example
/// Given the URI `/search?q=rust&page=2`:
/// ```rust
/// let req: Request<Incoming> = build_request_with_uri("/search?q=rust&page=2");
/// let params = parse_params(&req);
/// assert_eq!(params.q, Some("rust".to_string()));
/// assert_eq!(params.page, 2);
/// ```
///
/// Given the URI `/search`:
/// ```rust
/// let req: Request<Incoming> = build_request_with_uri("/search");
/// let params = parse_params(&req);
/// assert_eq!(params.q, None);
/// assert_eq!(params.page, 1);
/// ```
///
/// # Notes
/// - This function assumes the request URI contains valid UTF-8 encoded query strings.
/// - Be cautious when trusting user inputs, such as `page`, as they might cause unexpected behavior.
///
fn parse_params(req: &Request<Incoming>) -> QueryParams {
    let mut p = QueryParams { page: 1, ..Default::default() };
    if let Some(query) = req.uri().query() {
        for (k, v) in form_urlencoded::parse(query.as_bytes()) {
            match k.as_ref() {
                "q" => p.q = Some(v.into_owned()),
                "page" => {
                    if let Ok(n) = v.parse::<usize>() { p.page = n.max(1); }
                },
                _ => {}
            }
        }
    }
    p
}

/// Generates an HTML header string with a specified title.
///
/// This function constructs a reusable HTML `head` and `body` portion for a web page.
/// It includes metadata, a title, inline CSS for basic styling, and a formatted logo
/// placeholder at the top left, followed by a heading displaying the provided title.
///
/// # Arguments
///
/// * `title` - A string slice that holds the title of the page.
///
/// # Returns
///
/// * Returns a `String` which represents the initial portion of an HTML document,
///   including:
///   - `<!DOCTYPE html>` declaration.
///   - Metadata with UTF-8 character encoding.
///   - An inline `style` block with CSS for basic page layout and design (e.g., table borders,
///     typography, logo positioning, etc.).
///   - A placeholder `<div>` for the logo with an `<img>` tag pointing to `/logo.png`.
///   - An `<h2>` element displaying the provided title.
///
/// # Example
///
/// ```
/// let header = html_header("My Web Page");
/// println!("{}", header);
/// ```
///
/// Output:
///
/// ```html
/// <!DOCTYPE html><html><head><meta charset="utf-8"><title>My Web Page</title><style>
/// body{font-family:Arial,Helvetica,sans-serif;margin:20px}
/// table{border-collapse:collapse;width:100%}
/// th,td{border:1px solid #ddd;padding:8px}
/// th{background:#f4f4f4;text-align:left}
/// form input{margin:0 8px 8px 0;padding:6px}
/// nav a{margin-right:12px}
/// .logo{display:flex;justify-content:flex-start;margin:16px 0}
/// </style></head><body>
/// <div class="logo"><img src="/logo.png" alt="Logo" style="max-width:240px;height:auto"></div>
/// <h2>My Web Page</h2>
/// ```
///
/// Notes:
/// - The output of this function is not a complete HTML document. Additional content and a closing tag
///   for `<body>` and `<html>` should be appended to the result for a fully functional HTML page.
fn html_header(title: &str) -> String {
    let mut s = String::new();
    s.push_str("<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>");
    s.push_str(title);
    s.push_str("</title><style>body{font-family:Arial,Helvetica,sans-serif;margin:20px;background-image:linear-gradient(rgba(255,255,255,0.75), rgba(255,255,255,0.75)), url('/logo.png');background-repeat:no-repeat;background-position:center top;background-size:640px auto;background-attachment:fixed}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ddd;padding:8px}th{background:#f4f4f4;text-align:left}form input{margin:0 8px 8px 0;padding:6px}nav a{margin-right:12px}</style></head><body>");
    // Logo moved to background; no inline logo element
    s.push_str("<h2>");
    s.push_str(title);
    s.push_str("</h2>");
    s
}

fn opt_str(s: &Option<String>) -> &str { s.as_deref().unwrap_or("") }

/// Renders an HTML search form as a `String`.
///
/// This function generates a form using the `GET` method with an action
/// pointing to the `/search` endpoint. The form includes a single text input
/// field for query input (`q`), along with a submit button. The value of the
/// text input is pre-filled with the value of the query parameter `q`, if
/// provided. The input value is escaped to ensure HTML safety.
///
/// # Arguments
///
/// * `p` - A reference to a `QueryParams` struct that contains query parameters,
///          including the optional `q` parameter for the search input value.
///
/// # Returns
///
/// Returns a `String` containing the HTML of the form.
///
/// # Example
///
/// ```
/// let params = QueryParams { q: Some(String::from("example search")) };
/// let form_html = render_form(&params);
/// println!("{}", form_html);
/// ```
///
/// The above example will render the following HTML:
///
/// ```html
/// <form method="get" action="/search">
/// <input type="text" name="q" placeholder="Search filename, parent or checksum" value="example search">
/// <button type="submit">Search</button>
/// </form>
/// ```
fn render_form(p: &QueryParams) -> String {
    format!(
        "<form method=\"get\" action=\"/search\">\
         <input type=\"text\" name=\"q\" size=\"40\" placeholder=\"Search filename, parent or checksum\" value=\"{}\">\
         <button type=\"submit\">Search</button>\
         </form>",
        html_escape::encode_text(opt_str(&p.q)),
    )
}

/// Encodes query parameters and page number into a URL query string.
///
/// This function takes a reference to `QueryParams` and a `page` number as input,
/// and constructs a URL query string in the format `/search?q=query_value&page=page_number`.
/// Both parameter keys and values are URL-encoded to ensure they are safe for URLs.
///
/// # Arguments
///
/// * `p` - A reference to a `QueryParams` struct containing the query parameters.
///         - `q`: An optional string holding the main query term.
/// * `page` - A `usize` representing the page number to include in the query string.
///
/// # Returns
///
/// Returns a `String` representing the encoded query string, starting with `/search`
/// and containing the parameters `q` and `page` in URL-encoded format.
///
/// # Example
///
/// ```
/// struct QueryParams {
///     q: Option<String>,
/// }
///
/// let params = QueryParams { q: Some("rust programming".to_string()) };
/// let page_number = 2;
/// let encoded_url = encode_query(&params, page_number);
/// assert_eq!(encoded_url, "/search?q=rust%20programming&page=2");
/// ```
fn encode_query(p: &QueryParams, page: usize) -> String {
    let mut pairs: Vec<(String, String)> = vec![];
    if let Some(v) = &p.q { pairs.push(("q".to_string(), v.clone())); }
    pairs.push(("page".to_string(), page.to_string()));
    let qs: String = pairs
        .into_iter()
        .map(|(k, v)| format!("{}={}", urlencoding::encode(&k), urlencoding::encode(&v)))
        .collect::<Vec<_>>()
        .join("&");
    format!("/search?{}", qs)
}

/// Handles the HTTP GET request for the index route.
///
/// This function redirects the user to the search page (`/search`)
/// without any filters applied. This is intended to display the
/// first 100 entries as the default behavior.
///
/// # Parameters
///
/// - `_state`: A reference to the `FrontendState` instance.
///   This parameter is not used in this handler but is included
///   for consistency with other handlers that might need application state.
///
/// - `_req`: The incoming HTTP request of type `Request<Incoming>`.
///   This parameter is provided automatically but is not used directly in the handler.
///
/// # Returns
///
/// A `Response<Full<Bytes>>` object with the following properties:
/// - HTTP status code: `302 Found`
/// - Location header: `/search`
/// - Empty response body.
///
/// # Panics
///
/// This function can panic if the construction of the response fails,
/// although this situation is unlikely.
async fn handle_index(_state: &FrontendState, _req: Request<Incoming>) -> Response<Full<Bytes>> {
    // Redirect to the search page without filters to show the first 100 entries by default
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, "/search")
        .body(Full::new(Bytes::from("")))
        .unwrap()
}

/// Constructs a SQL query and its associated parameters for searching in a "files" database table.
///
/// # Parameters
///
/// - `p`: A reference to a `QueryParams` structure that contains search parameters.
///        Specifically, this structure may include:
///        - `q`: An optional query string to match against certain fields in the "files" table.
///
/// # Returns
///
/// A tuple containing:
/// 1. A `String` representing the SQL query that includes a base query and additional WHERE clauses,
///    depending on the provided parameters.
/// 2. A `Vec<Box<dyn ToSql>>`, which is a collection of parameter values to be safely bound to the
///    SQL query at runtime.
///
/// # Behavior
///
/// - The function starts with a base SQL query: `"FROM files WHERE 1=1"`.
/// - If the `q` field of `QueryParams` is `Some`, the function appends a condition to the SQL
///   query to search for the provided string (`q`) in the following fields:
///   - `filename`
///   - `parent`
///   - `md5`
///   - `sha1`
///   - `sha256`.
/// - It also generates corresponding parameterized values (using `LIKE`) for these fields,
///   ensuring safe and SQL injection-preventive execution using prepared statements.
/// - The `q` field, if provided, is wrapped with wildcard `%` to enable partial matches.
///
/// # Example
///
/// ```
/// // Suppose we have a struct `QueryParams` and the following instance:
/// let params = QueryParams { q: Some("example".to_string()) };
///
/// // Call the function to build the SQL query.
/// let (query, bind_params) = build_sql(&params);
///
/// // The resulting query would look like:
/// // "FROM files WHERE 1=1 AND (filename LIKE ? OR parent LIKE ? OR md5 LIKE ? OR sha1 LIKE ? OR sha256 LIKE ?)"
///
/// // The `bind_params` would contain the following values:
/// // ["%example%", "%example%", "%example%", "%example%", "%example%"]
/// ```
///
/// # Notes
///
/// - The `ToSql` trait is typically used in Rust database libraries (e.g., `rusqlite`) to handle
///   parameterized SQL queries.
/// - The function assumes the caller will prepend a `SELECT` statement to the returned query string.
fn build_sql(p: &QueryParams) -> (String, Vec<Box<dyn ToSql>>) {
    let mut sql = String::from("FROM files WHERE 1=1");
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(q) = &p.q {
        sql.push_str(" AND (filename LIKE ? OR parent LIKE ? OR md5 LIKE ? OR sha1 LIKE ? OR sha256 LIKE ?)");
        let like = format!("%{}%", q);
        params.push(Box::new(like.clone()));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like.clone()));
        params.push(Box::new(like));
    }

    (sql, params)
}

/// Handles HTTP requests for the search functionality.
///
/// This asynchronous function processes incoming search requests, queries the database,
/// and dynamically generates an HTML response containing the search results table along
/// with pagination. It ensures proper error handling for database operations and escape
/// encoding for HTML output to prevent issues like SQL injection or cross-site scripting (XSS).
///
/// # Arguments
/// * `state` - A reference to the `FrontendState` containing application state, such as the database path.
/// * `req` - The incoming HTTP request to be handled.
///
/// # Returns
/// * `Response<Full<Bytes>>` - A fully built HTTP response containing the HTML body or an error message.
///
/// # Workflow
/// 1. Extract parameters from the request using the `parse_params` function.
/// 2. Open a SQLite database connection using the database path available in the `state`.
/// 3. Build the SQL query string and parameters based on input.
///
///    - Count the total number of rows matching the search criteria.
///    - Paginate the results by calculating the appropriate offset and limit for the current page.
///
/// 4. Execute the SQL query to retrieve rows matching the search criteria.
/// 5. Construct an HTML page:
///    - Generates an overview of search statistics (total matches, current page, total pages).
///    - Creates a table displaying the search results with fields like ID, parent, filename,
///      track, sector, length, and cryptographic hash values (MD5, SHA1, SHA256).
///    - Adds navigation links for pagination.
/// 6. Return an HTTP response with the generated HTML.
///
/// # Edge Cases
/// - If there's an issue opening the database, preparing SQL statements, or executing queries, an
///   HTML error response with an appropriate message and HTTP status (e.g., `500 Internal Server Error`) is returned.
/// - Pagination gracefully handles cases with zero results or out-of-bounds pages.
///
/// # Dependencies
/// - Relies on the `rusqlite` crate for SQLite database operations.
/// - Uses helper functions like `parse_params`, `build_sql`, `html_header`, `render_form`, `html_error`,
///   `encode_query`, and the `html_escape` crate for utility operations.
///
/// # Example
/// ```
/// async fn handle_request(req: Request<Incoming>) -> Response<Full<Bytes>> {
///     let state = FrontendState { db_path: String::from("path/to/db.sqlite") };
///     handle_search(&state, req).await
/// }
/// ```
/// This example showcases how to invoke the search handling function in a web server context.
///
/// # Note
/// - Use appropriate safeguards before deploying this function in production, such as validating user inputs
///   and protecting against injection attacks.
async fn handle_search(state: &FrontendState, req: Request<Incoming>) -> Response<Full<Bytes>> {
    let p = parse_params(&req);
    let db_path = &*state.db_path;
    let conn = match Connection::open(db_path) { Ok(c) => c, Err(e) => return html_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("DB open error: {}", e)) };

    let (where_sql, params) = build_sql(&p);

    // Count total
    let mut count_stmt = match conn.prepare(&format!("SELECT COUNT(*) {}", where_sql)) {
        Ok(s) => s, Err(e) => return html_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("prepare count: {}", e))
    };
    let total: i64 = match count_stmt.query_row(rusqlite::params_from_iter(params.iter().map(|b| &**b)), |row| row.get(0)) { Ok(n) => n, Err(e) => return html_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("count failed: {}", e)) };

    let page_size: i64 = 100;
    let page = if p.page == 0 { 1 } else { p.page } as i64;
    let offset = (page - 1) * page_size;

    // Query page
    let mut stmt = match conn.prepare(&format!(
        "SELECT id, parent, filename, track, sector, length, md5, sha1, sha256 {} ORDER BY id LIMIT ? OFFSET ?",
        where_sql
    )) { Ok(s) => s, Err(e) => return html_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("prepare select: {}", e)) };

    let mut bind: Vec<&dyn ToSql> = params.iter().map(|b| &**b as &dyn ToSql).collect();
    bind.push(&page_size);
    bind.push(&offset);

    let mut rows = match stmt.query(rusqlite::params_from_iter(bind.into_iter())) {
        Ok(r) => r, Err(e) => return html_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("query failed: {}", e))
    };

    let mut html = String::new();
    html.push_str(&html_header("CBM Parser - Results"));
    html.push_str(&render_form(&p));

    html.push_str(&format!("<p>Total matches: {}. Page {} of {}.</p>", total, page, ((total + page_size - 1) / page_size).max(1)));
    html.push_str("<table><thead><tr><th>ID</th><th>Parent</th><th>Filename</th><th>Track</th><th>Sector</th><th>Length</th><th>MD5</th><th>SHA1</th><th>SHA256</th></tr></thead><tbody>");

    while let Ok(Some(row)) = rows.next() {
        let id: i64 = row.get(0).unwrap_or_default();
        let parent: String = row.get(1).unwrap_or_default();
        let filename: String = row.get(2).unwrap_or_default();
        let track: i64 = row.get(3).unwrap_or_default();
        let sector: i64 = row.get(4).unwrap_or_default();
        let length: i64 = row.get(5).unwrap_or_default();
        let md5: String = row.get(6).unwrap_or_default();
        let sha1: String = row.get(7).unwrap_or_default();
        let sha256: String = row.get(8).unwrap_or_default();
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td><code>{}</code></td><td><code>{}</code></td></tr>",
            id,
            html_escape::encode_text(&parent),
            html_escape::encode_text(&filename),
            track,
            sector,
            length,
            html_escape::encode_text(&md5),
            html_escape::encode_text(&sha1),
            html_escape::encode_text(&sha256)
        ));
    }
    html.push_str("</tbody></table>");

    let total_pages = ((total + page_size - 1) / page_size).max(1);
    html.push_str("<nav>");
    if page > 1 { html.push_str(&format!("<a href=\"{}\">&laquo; Prev</a>", encode_query(&p, (page-1) as usize))); }
    if page < total_pages { html.push_str(&format!("<a href=\"{}\">Next &raquo;</a>", encode_query(&p, (page+1) as usize))); }
    html.push_str("</nav>");

    html.push_str("</body></html>");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(html)))
        .unwrap()
}

/// Generates an HTML-formatted error response with the provided status code and message.
///
/// # Parameters
///
/// - `code`: The HTTP status code to be used in the response. This represents the type of error.
/// - `msg`: A descriptive error message that will be escaped and included in the HTML response body.
///
/// # Returns
///
/// A `Response` object that contains:
/// - The provided HTTP status code.
/// - A "Content-Type" header set to `"text/html; charset=utf-8"`.
/// - A formatted HTML body displaying the status code and the escaped error message.
///
/// The body of the response is created by combining the status code and the escaped error message,
/// ensuring that any special characters in the message are properly HTML-escaped to prevent issues
/// such as HTML injection.
///
/// # Examples
///
/// ```
/// use http::{Response, StatusCode};
/// use bytes::Bytes;
/// use hyper::body::Full;
///
/// let response = html_error(StatusCode::NOT_FOUND, "The requested resource was not found.");
///
/// assert_eq!(response.status(), StatusCode::NOT_FOUND);
/// assert_eq!(response.headers()["Content-Type"], "text/html; charset=utf-8");
/// assert_eq!(
///     response.body(),
///     &Full::new(Bytes::from("404 Not Found: The requested resource was not found."))
/// );
/// ```
///
/// # Errors
///
/// The method will panic if the `Response` builder fails to construct the response, which could occur
/// due to invalid parameters or other internal errors.
fn html_error(code: StatusCode, msg: &str) -> Response<Full<Bytes>> {
    let body = format!("{}: {}", code, html_escape::encode_text(msg));
    Response::builder()
        .status(code)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Full::new(Bytes::from(body)))
        .unwrap()
}

/// Asynchronously serves the logo PNG file as an HTTP response.
///
/// This function reads the "logo.png" file located in the "assets" directory
/// and constructs an HTTP response containing the image data. It uses the Tokio
/// asynchronous runtime for non-blocking file operations.
///
/// # Returns
///
/// This function returns an HTTP response, which:
///
/// - Contains the logo image with a MIME type of "image/png" and a status code
///   of `200 OK` if the file is successfully read.
/// - Responds with an HTML error page with a status code of `404 Not Found` if
///   the logo file cannot be located or read.
///
/// # Errors
///
/// - If the "logo.png" file is not found or cannot be read, a `404 Not Found`
///   error response is returned.
///
/// # Dependencies
///
/// - Assumes the presence of a "logo.png" file in the "assets" directory at runtime.
/// - Uses the `tokio::fs::read` function for asynchronous file reading.
/// - Utilizes helper functions like `html_error` to format error responses. Ensure
///   that this helper function is defined and accessible in your code.
///
/// # Example
///
/// Here is an example of how this function might respond in a real HTTP server:
///
/// ```no_run
/// async fn handle_request() -> Response<Full<Bytes>> {
///     serve_logo().await
/// }
///
/// // When the "logo.png" file is present:
/// // HTTP/1.1 200 OK
/// // Content-Type: image/png
/// // ...[binary PNG data]
///
/// // If the file is missing:
/// // HTTP/1.1 404 NOT FOUND
/// // Content-Type: text/html
/// // ...[HTML error content]
/// ```
///
/// Ensure that the "assets" directory is correctly structured and that it contains
/// the "logo.png" file for this function to function properly.
async fn serve_logo() -> Response<Full<Bytes>> {
    let path = Path::new("assets").join("logo.png");
    match tokio::fs::read(&path).await {
        Ok(bytes_vec) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "image/png")
                .body(Full::new(Bytes::from(bytes_vec)))
                .unwrap()
        }
        Err(_) => html_error(StatusCode::NOT_FOUND, "Logo not found"),
    }
}

#[derive(Clone)]
struct FrontendService {
    state: FrontendState,
}

impl Service<Request<Incoming>> for FrontendService {
    type Response = Response<Full<Bytes>>;
    type Error = Infallible;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let path = req.uri().path().to_string();
        let state = self.state.clone();
        Box::pin(async move {
            let resp = match (req.method().as_str(), path.as_str()) {
                ("GET", "/") => handle_index(&state, req).await,
                ("GET", "/search") => handle_search(&state, req).await,
                ("GET", "/logo.png") => serve_logo().await,
                _ => html_error(StatusCode::NOT_FOUND, "Not Found"),
            };
            Ok(resp)
        })
    }
}

/// Starts a Hyper-based frontend HTTP server.
///
/// The server listens for incoming HTTP requests on `127.0.0.1:3000` and serves connections
/// using HTTP/1.1. It manages state encapsulated in `FrontendState`, which contains the
/// provided database path.
///
/// # Arguments
///
/// * `db_path` - A string slice referencing the path to the database used by the frontend server.
///
/// # Returns
///
/// This function returns a `Result<(), Box<dyn std::error::Error + Send + Sync>>`, indicating
/// success or failure. Possible errors include failures in binding to the socket, accepting new
/// connections, or serving HTTP requests due to underlying Hyper errors.
///
/// # Behavior
///
/// * The server binds to the `127.0.0.1:3000` address and runs indefinitely in an asynchronous loop.
/// * Each incoming connection spawns a new task to handle the connection using `tokio::task::spawn`.
/// * The server processes HTTP requests using the provided `FrontendService`.
///
/// # Errors
///
/// This function propagates errors that may occur during:
/// * Parsing the socket address.
/// * Binding to the specified address.
/// * Accepting incoming TCP connections.
/// * Serving HTTP connections via Hyper.
///
/// # Example
///
/// ```rust,ignore
/// #[tokio::main]
/// async fn main() {
///     if let Err(e) = start_frontend_server("/path/to/db").await {
///         eprintln!("Server failed with error: {}", e);
///     }
/// }
/// ```
///
/// # Notes
///
/// Make sure the provided database path exists and the application has appropriate
/// permissions to access it. The server currently only supports HTTP/1.1 connections
/// and listens on `127.0.0.1` for local access only.
pub async fn start_frontend_server(db_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let state = FrontendState { db_path: Arc::new(db_path.to_string()) };

    let listener = TcpListener::bind(addr).await?;
    println!("Starting Hyper frontend on http://{}", addr);

    loop {
        let (stream, _) = listener.accept().await?;
        let svc = FrontendService { state: state.clone() };
        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new().serve_connection(hyper_util::rt::TokioIo::new(stream), hyper::service::service_fn(move |req| {
                let svc = svc.clone();
                async move { svc.call(req).await }
            })).await {
                eprintln!("connection error: {}", e);
            }
        });
    }
}
