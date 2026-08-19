* Architecture
  * [Ingestion Tier](Ingestion_Tier.adoc)
  * [Kafka Tier](./Kafka_Tier.md)
  * [Consumer Tier](./Consumer_Tier.md)
  * [DB Tier](./SQL_Tier.md)
  * [Redis Tier](./Redis_Tier.md)
  * [Metrics](./Metrics.md)

# Architecture
- Ingestion Tier(3..200 pods), Consumer Tier(3..50 pods) scaled independently
- Service to scale horizontally. The Rust API is stateless and Tokio-based
```
Ingestion Layer       Kafka              Consumer Layer
---------------     ---------            -------------
100 instances   →   100 partitions   →   50 workers   →   DB
```

```mermaid
flowchart LR
    DNS["DNS<br>Route 53"]
    C1["Collecto1"]
    C2["Collector2"]
    GLB["Global LB<br>Regional LB"]
    
    subgraph Datacenter
        APIGW["API GW<br/>nginx<br>TLS termination"]
        LB["ELB/ALB"]
        subgraph JIMS Server
            subgraph API["Ingestion Tier(100-200 pods)"]
                KLB["k8s<br>Ingress LB"]
                P1["Pod 1<br/>Tokio Runtime<br> Kafka Producer"]
                P2["Pod 2<br/>Tokio Runtime<br> Kafka Producer"]
                PN["Pod 200<br/>Tokio Runtime<br> Kafka Producer"]
            end
            K[["Kafka<br>Tier<br><br> Partitions<br>P0 P1 P2...P99"]]
            subgraph Workers["Consumer Tier (50-100 pods)"]
                W1["Worker Pod 1<br><br>→ P0-P9"]
                W2["Worker Pod 2<br><br>→ P10-P19"]
                W3["Worker Pod 3<br><br>→ P20-P29"]
            end
            redis[["Redis<br>Immediate IP Query"]]
            subgraph DBT["DB Tier"]
                DB[["SQL<br>(AWS Aurora)"]]
                RR[["Read Replicas"]]
            end
            srx["SRX"]
        end
    end

    C1 -->|hostname|DNS
    DNS -->|IP|C1

    C1 -->|Batched JSON| GLB
    C2 -->|Batched JSON| GLB
    GLB -->|HTTPS| APIGW
    APIGW -->|HTTP| LB
    LB --> KLB
    KLB --> P1
    KLB --> P2
    KLB --> PN

    API -->|Topic:identity-events| K
    K -->|consumer group| Workers

    Workers --> DBT
    Workers --> redis
    DB <--> RR

    RR --> srx
    redis --> srx
```
