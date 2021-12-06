use warp::Filter;

pub async fn start_server() {
    let hello = warp::path!("hello" / String).map(|x| x);
    warp::serve(hello).run(([127, 0, 0, 1], 3030)).await;
}
