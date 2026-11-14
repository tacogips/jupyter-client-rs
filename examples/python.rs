use jupyter_client::*;

#[tokio::main]
async fn main() {
    let client = JupyterClient::default();
    let resp = client.get_root_contents().await.unwrap();

    println!("{resp:?}");
    let resp = client.get_sessions().await.unwrap();
    println!("{resp:?}");

    let resp = client
        .put_contents("test.ipynb", vec!["12 + 24".to_string()].into())
        .await
        .unwrap();
    println!("{resp:?}");

    let kernels = client.get_running_kernels().await.unwrap();
    println!("kernels:{kernels:?}");
    let resp = kernels.iter().find(|each| each.name == "python3");
    if resp.is_none() {
        client
            .start_kernel(KernelPostRequest {
                name: "python3".to_string(),
                path: None,
            })
            .await
            .unwrap();
    }

    let kernels = client.get_running_kernels().await.unwrap();
    let kernel = kernels.iter().find(|each| each.name == "python3").unwrap();
    let kernsl_cli = client.new_kernel_client(&kernel).unwrap();

    let resp = kernsl_cli.run_code("12 * 22".into(), None).await;
    println!("{resp:?}");

    let resp = kernsl_cli
        .run_code(
            "import matplotlib.pyplot as plt\nplt.plot([1,2,3],[2,4,3])".into(),
            None,
        )
        .await;
    println!("--- {:?}", resp.unwrap());
}
