This is a minimum example to reproduce a presumed [bug](https://github.com/tokio-rs/tracing/issues/2493) in one of the opentelemetry crates.

Can be tested against jaegertracing/all-in-one docker container:

1. Run jaeger docker container
    ```
    docker run -d --name jaeger \     
                                  -e COLLECTOR_ZIPKIN_HTTP_PORT=9411 \
                                  -p 5775:5775/udp \
                                  -p 6831:6831/udp \
                                  -p 6832:6832/udp \
                                  -p 5778:5778 \
                                  -p 16686:16686 \
                                  -p 14268:14268 \
                                  -p 9411:9411 \
                                  jaegertracing/all-in-one:1.6
    ```

2. Modify this code (`main.rs`) to have correct Jaeger Agent addresses for your host

3. Execute this program

4. Look at http://localhost:16686/search  
   (or just watch network traffic on udp port 6831 with wireshark)

5. Observe no spans arriving :(

6. Modify this code to include the line
   ```
   let _ = global::set_tracer_provider(tracer_provider);
   ```
   in the FIXME section of `main.rs`.

7. Repeat the experiment and see: now traces arrive
