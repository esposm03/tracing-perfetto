use tokio::runtime::Handle;
use tracing::{field, info, span, Level};
use tracing_perfetto::PerfettoLayer;
use tracing_subscriber::fmt::format::Format;
use tracing_subscriber::{fmt, layer::SubscriberExt, Registry};

#[tokio::test]
async fn write() -> anyhow::Result<()> {
    let file = std::path::Path::new("test.pftrace");
    let perfetto_layer = PerfettoLayer::new(std::sync::Mutex::new(std::fs::File::create(&file)?))
        .with_debug_annotations(true)
        .with_screenshots(true);

    let fmt_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .event_format(Format::default().with_thread_ids(true))
        .with_span_events(fmt::format::FmtSpan::FULL);

    let subscriber = Registry::default().with(fmt_layer).with(perfetto_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    // screenshot();
    info!(?file, "start");

    let demo_span = span!(
        Level::TRACE,
        "demo_span",
        regular_arg = "Arg data",
        extra_arg = field::Empty
    );
    let _enter = demo_span.enter();

    info!("in span");
    sync_fn(1);
    let handle = Handle::current();
    let t = std::thread::spawn(move || {
        handle.spawn(async_fn());
    });
    t.join().unwrap();

    _ = tokio::spawn(async_fn()).await;
    demo_span.record("extra_arg", "Some Extra Data");
    demo_span.record("regular_arg", "New Arg Data");
    Ok(())
}

#[tracing::instrument(fields(perfetto = true))]
fn sync_fn(i: i32) {
    info!("inside function");
    sync_inner(i + 1);
}

#[tracing::instrument(skip_all, level = "trace", fields(perfetto = true))]
fn sync_inner(x: i32) {
    info!(x, "inner");
}

#[tracing::instrument]
async fn async_fn() {
    info!(perfetto = true, "test");
    async_inner().await;
}

#[tracing::instrument]
async fn async_inner() {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}

// fn screenshot() {
//     event!(
//         tracing::Level::INFO,
//         screenshot_jpeg_bytes = include_bytes!("../cat.jpg").as_slice(),
//         "Test",
//     );
// }
