# User Journeys - maidos-shared

## Journey J-001: Developer Integrates Token-Based Authentication

**Actor**: Product Engineer (MAIDOS-Driver team)
**Goal**: Add capability-based authentication to driver installation flow
**Precondition**: Rust project with Cargo.toml

### Steps

1. **Add Dependency**
   ```toml
   [dependencies]
   maidos-auth = "0.2"
   ```

2. **Create Token Issuer**
   ```rust
   use maidos_auth::{TokenIssuer, Capability};

   let secret = b"my-256-bit-secret-key...";
   let issuer = TokenIssuer::new(secret);
   ```

3. **Issue Token with Capabilities**
   ```rust
   let token = issuer.issue(
       "user_123",
       vec![Capability::WriteDriver, Capability::ReadHardware],
       Duration::from_hours(24)
   )?;
   ```

4. **Verify Token in Request Handler**
   ```rust
   let claims = issuer.verify(&token)?;
   if claims.has_capability(Capability::WriteDriver) {
       install_driver()?;
   }
   ```

5. **Handle Expiration**
   ```rust
   match issuer.verify(&token) {
       Err(AuthError::TokenExpired) => return refresh_token(),
       Err(e) => return Err(e),
       Ok(claims) => proceed(claims),
   }
   ```

**Outcome**: Driver installation requires valid token with WriteDriver capability. Token verification completes in < 1ms.

**Acceptance Criteria**: AC-001, AC-002, AC-004

---

## Journey J-002: Application Uses Multi-Provider LLM Chat

**Actor**: Product Engineer (MAIDOS-CodeQC team)
**Goal**: Add AI code review feature supporting multiple LLM providers
**Precondition**: Rust project with async runtime (tokio)

### Steps

1. **Add Dependency**
   ```toml
   [dependencies]
   maidos-llm = "0.2"
   tokio = { version = "1.32", features = ["full"] }
   ```

2. **Create Provider Router**
   ```rust
   use maidos_llm::{Router, ProviderType, create_provider, RoutingStrategy};

   let openai = create_provider(ProviderType::OpenAI, Some("sk-..."), None)?;
   let claude = create_provider(ProviderType::Anthropic, Some("sk-ant-..."), None)?;
   let ollama = create_provider(ProviderType::Ollama, None, None)?;

   let router = Router::new(vec![openai, claude, ollama])
       .with_strategy(RoutingStrategy::Fallback);
   ```

3. **Send Code Review Request**
   ```rust
   use maidos_llm::{CompletionRequest, Message, Role};

   let request = CompletionRequest::new(
       "gpt-4o",
       vec![
           Message::system("You are a code reviewer."),
           Message::user("Review this code: fn main() { ... }"),
       ],
   );

   let response = router.route(request).await?;
   println!("Review: {}", response.text);
   ```

4. **Handle Streaming Response**
   ```rust
   let mut stream = router.route_streaming(request).await?;

   while let Some(chunk) = stream.next().await {
       match chunk? {
           MaidosStreamItem::TextDelta(text) => print!("{}", text),
           MaidosStreamItem::Done(usage) => println!("\nTokens: {}", usage.total_tokens),
           _ => {}
       }
   }
   ```

5. **Enforce Budget Limits**
   ```rust
   let router = Router::new(providers)
       .with_budget(BudgetConfig {
           daily_limit: 100.0,  // $100/day
           per_request_limit: 5.0,
       });
   ```

**Outcome**: Code review feature uses OpenAI by default, falls back to Claude if OpenAI fails, and falls back to local Ollama if both cloud providers are unavailable. Daily spending capped at $100.

**Acceptance Criteria**: AC-009, AC-010, AC-011, AC-012, AC-013

---

## Journey J-003: Service Connects to Event Bus for Pub/Sub

**Actor**: Service Engineer (MAIDOS-Forge distributed build system)
**Goal**: Receive job notifications via event bus
**Precondition**: ZeroMQ installed on host

### Steps

1. **Add Dependency**
   ```toml
   [dependencies]
   maidos-bus = "0.2"
   tokio = { version = "1.32", features = ["full"] }
   ```

2. **Start Publisher (Job Scheduler)**
   ```rust
   use maidos_bus::{Publisher, BusMessage};

   let mut publisher = Publisher::new("tcp://*:5555").await?;

   let msg = BusMessage::new("job.created", serde_json::json!({
       "job_id": "j_001",
       "target": "linux-x64",
   }));

   publisher.publish(msg).await?;
   ```

3. **Start Subscriber (Build Worker)**
   ```rust
   use maidos_bus::Subscriber;

   let mut subscriber = Subscriber::new("tcp://localhost:5555")
       .subscribe("job.created")
       .await?;

   while let Some(msg) = subscriber.receive().await? {
       println!("Received job: {:?}", msg.payload);
       process_job(msg.payload)?;
   }
   ```

4. **Handle Disconnection**
   ```rust
   loop {
       match subscriber.receive().await {
           Ok(Some(msg)) => process_job(msg)?,
           Ok(None) => continue,
           Err(BusError::Disconnected) => {
               println!("Reconnecting...");
               subscriber.reconnect().await?;
           }
           Err(e) => return Err(e),
       }
   }
   ```

5. **Filter by Topic Prefix**
   ```rust
   let subscriber = Subscriber::new("tcp://localhost:5555")
       .subscribe("job.")  // Matches job.created, job.completed, job.failed
       .await?;
   ```

**Outcome**: Build workers receive job notifications in real-time. If connection drops, workers automatically reconnect within 30 seconds.

**Acceptance Criteria**: AC-005, AC-006, AC-007, AC-008

---

## Journey J-004: C# Application Consumes Rust Library via FFI

**Actor**: UI Engineer (MAIDOS-Driver WPF application)
**Goal**: Call Rust token verification from C# UI
**Precondition**: Visual Studio project, maidOS_shared.dll compiled

### Steps

1. **Add DLL Reference**
   ```xml
   <ItemGroup>
     <None Include="..\..\maidos-shared\target\release\maidos_shared.dll">
       <CopyToOutputDirectory>PreserveNewest</CopyToOutputDirectory>
     </None>
   </ItemGroup>
   ```

2. **Define P/Invoke Binding**
   ```csharp
   using System.Runtime.InteropServices;

   public static class MaidosAuth {
       [DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
       public static extern IntPtr maidos_auth_verify_token(
           [MarshalAs(UnmanagedType.LPUTF8Str)] string token,
           [MarshalAs(UnmanagedType.LPUTF8Str)] string secret
       );

       [DllImport("maidos_shared.dll", CallingConvention = CallingConvention.Cdecl)]
       public static extern void maidos_auth_free_result(IntPtr result);
   }
   ```

3. **Call Rust Function**
   ```csharp
   string token = GetTokenFromUser();
   IntPtr resultPtr = MaidosAuth.maidos_auth_verify_token(token, "my-secret");

   if (resultPtr == IntPtr.Zero) {
       MessageBox.Show("Invalid token");
       return;
   }

   string userId = Marshal.PtrToStringUTF8(resultPtr);
   MaidosAuth.maidos_auth_free_result(resultPtr);

   MessageBox.Show($"Authenticated: {userId}");
   ```

**Outcome**: C# UI verifies authentication tokens using Rust crypto library without reimplementing JWT logic.

**Acceptance Criteria**: AC-001, AC-002

---

## Journey J-005: Engineer Debugs Configuration Loading Issue

**Actor**: DevOps Engineer
**Goal**: Troubleshoot why application is not loading custom config file
**Precondition**: Application logs available

### Steps

1. **Enable Debug Logging**
   ```rust
   use maidos_log::init_logger;
   init_logger(LogLevel::Debug)?;
   ```

2. **Check Config Load Path**
   ```rust
   use maidos_config::ConfigLoader;

   let loader = ConfigLoader::new();
   match loader.load("app.toml") {
       Ok(config) => println!("Loaded: {:?}", config),
       Err(e) => eprintln!("Failed to load: {}", e),
   }
   ```

3. **Inspect Log Output**
   ```json
   {
     "level": "DEBUG",
     "message": "Attempting to load config from app.toml",
     "timestamp": "2026-02-13T12:34:56Z"
   }
   {
     "level": "ERROR",
     "message": "Config file not found: app.toml",
     "path": "/expected/path/app.toml",
     "timestamp": "2026-02-13T12:34:56Z"
   }
   ```

4. **Verify File Path**
   ```bash
   ls -la /expected/path/app.toml
   # File does not exist
   ```

5. **Correct Configuration**
   ```bash
   cp config/app.toml /expected/path/app.toml
   ```

**Outcome**: Engineer identifies missing config file from structured log output and corrects deployment.

**Acceptance Criteria**: AC-017

---

## Journey J-006: Application Streams LLM Response to UI

**Actor**: Frontend Engineer (MAIDOS-Office AI assistant)
**Goal**: Display LLM response progressively as it generates
**Precondition**: UI supports async updates

### Steps

1. **Setup Streaming Provider**
   ```rust
   let provider = create_provider(ProviderType::OpenAI, Some(api_key), None)?;
   let request = CompletionRequest::quick("Explain Rust ownership.");
   ```

2. **Stream to UI**
   ```rust
   let mut stream = provider.complete_streaming(request).await?;

   while let Some(item) = stream.next().await {
       match item? {
           MaidosStreamItem::TextDelta(text) => {
               ui_text_box.append(text);
               ui_text_box.refresh();
           }
           MaidosStreamItem::Done(usage) => {
               ui_status_bar.set_text(format!("Tokens: {}", usage.total_tokens));
           }
           _ => {}
       }
   }
   ```

**Outcome**: User sees LLM response appear word-by-word in real-time instead of waiting for full completion.

**Acceptance Criteria**: AC-011
