* Architecture
  * [Ingestion Tier](./Ingestion_Tier/README.md)
  * [Kafka Tier](./Kafka_Tier.md)
  * [Consumer Tier](./Consumer_Tier.md)
  * [DB Tier](./SQL_Tier.md)
  * [Redis Tier](./Redis_Tier.md)
  * [Metrics](./Metrics.md)
  * [Query Tier](./QueryAPI.md)
* [Architecture when ingest reaches 1.5M requests/sec](#architecture-when-ingest-reaches-15m-requestssec)

# Architecture
## Old
```
              |----------------- Server ------------|
			  |									    |
Collector-->  |rust  						   rust	----cache---->postgres
			  |task ---> |unbounded queue| --> task	|	  |			|
			  |										|	  |			|
			  |								rust   <------|---------|
			  |								task    |
			  |------------------------------/\-----|
											  |IP/user query
											  |
											  SRX firewall
```

## New
### Ingest Pods
```
1 pod(12000 req/sec), 5 pods(60k), 10 pods(100k), 125 pods(1.5M)
```

### Consumer Pods
```
30-40 pods(identity partition), 8-16 pods(session consumer), 4-8(catalog consumer)
```

```mermaid
flowchart LR
autonumber
    DNS["DNS<br>Route 53"]
    C1["Collecto1"]
    C2["Collector2"]
    GLB["Global LB<br>Regional LB"]
    
    subgraph Datacenter
        APIGW["API GW<br/>nginx<br>TLS termination"]
        LB["ELB/ALB"]
        subgraph JIMS Server
            subgraph API["Ingestion Tier(3-10 pods)"]
                KLB["k8s<br>Ingress LB"]
                P1["Pod 1<br/>Tokio Runtime<br> Kafka Producer"]
                P2["Pod 2<br/>Tokio Runtime<br> Kafka Producer"]
                PN["Pod 200<br/>Tokio Runtime<br> Kafka Producer"]
            end
            K[["Kafka<br>Tier<br><br> Partitions<br>P0 P1 P2...P99"]]
            subgraph Workers["Consumer Tier (3-10 pods)"]
                W1["Worker Pod 1<br><br>→ P0-P9"]
                W2["Pod 2<br><br>→ P10-P19"]
                W3["Pod 3<br><br>→ P20-P29"]
            end
            redis[["Redis<br>IP Query"]]
            subgraph DBT["DB Tier / AWS Aurora"]
                DB[["Master/Primary<br>shard1<br>4+ TB"]]
                DB1[["Master<br>shard2"]]
                DB2[["Master<br>shard3"]]
                RR[["Read Replicas<br>shard1"]]
                RR1[["Read Replicas<br>shard2"]]
                RR2[["Read Replicas<br>shard3"]]
            end
            QAPI["QueryAPI"]
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

    Workers -->|consumer<br>write to<br>master| redis
    redis -->|async copy| DB
    redis --> DB1
    redis --> DB2
    
    DB -->|sync| RR
    DB -->|sync| RR1
    DB -->|sync| RR2

    RR -->|Read from<br>Read Replica<br><br>if redis miss<br>10–30 s latency| QAPI
    QAPI -->|query| redis
    redis -->|response<br> p99 < 10 ms| QAPI
    
    QAPI --> srx
```

---

## Old vs New

||Old|New|
|---|---|---|
|Design|1 process does ingest, buffering, DB writes, and firewall queries|jobs are split, bounded, and scaled independently|
|1sec Stall|~100k extra items in RAM||
|10sec Stall|1 million. Memory grows until the process dies.On crash, everything in that queue is gone — it is not on disk||
