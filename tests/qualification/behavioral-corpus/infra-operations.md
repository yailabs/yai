# Servizio di inferenza interno — operazioni infrastrutturali

## Scopo e richiesta

L’operatore vuole usare Studio come postazione di controllo di uno stack tecnico:
YAI Host e Studio sulla postazione Exon, servizio YVEX e modello DeepSeek su DGX
Spark. Il lavoro concreto è rendere disponibile l’inferenza interna attraverso
YAI, con controllo degli accessi, osservazione del contesto e recupero verificabile.

Questo documento è una richiesta operativa, non un verbale di collaudo. Non
attribuisce lo stack a una società inventata, non dichiara clienti o SLA e non
attesta disponibilità corrente. Il ruolo aziendale è quello di un team Platform
Engineering che prepara un servizio interno per altri operatori tecnici.

## Storia del lavoro

La richiesta nasce dalla necessità di evitare una sequenza manuale dispersa tra
terminali, configurazione provider e interfaccia Studio. L’operatore deve poter
capire quale macchina ospita il servizio, quale modello usa, cosa può fare il Case
e perché una richiesta riesce o viene rifiutata.

Il percorso è: richiesta → inventario → preparazione del cambiamento →
autorizzazione → attivazione → verifica → consegna oppure rollback.
Il Workflow collegato contiene sette checkpoint umani. Un checkpoint scritto
non esegue comandi, non concede permessi e non prova il completamento del lavoro.
Le dipendenze e gli esiti effettivi restano nello stato e nella storia YAI.

## Componenti e dipendenze

| Componente | Ruolo previsto | Evidenza richiesta prima dell’uso |
|---|---|---|
| Exon | Postazione Studio e Host YAI | Identità macchina, versioni e disponibilità osservate |
| DGX Spark | Macchina del servizio YVEX | Identità verificata, accesso autorizzato, risorse disponibili |
| YVEX | Runtime del modello | Contratti pubblici, endpoint raggiungibile, stato osservato |
| DeepSeek V4 Flash | Modello richiesto | Identità esposta, rappresentazione e capacità qualificate |
| Provider YAI | Target governato condivisibile | Registrazione, qualifica e trust correnti |
| Case Compute | Utilizzo del target in questo Case | Binding e requisiti cognitivi qualificati |

L’operatore ha mostrato un catalogo con V4 Flash READY e un host YVEX avviato.
Sono osservazioni storiche riferite dall’operatore: non provano che il modello
sia oggi caricato né che il servizio sia raggiungibile. Nessun indirizzo macchina
o segreto è incluso nel materiale riproducibile.

## Autorità e confini

La policy documentale acquisita consente le sole operazioni dichiarate di
scoperta, ammissione e lettura dei materiali nel perimetro. Non autorizza avvio
remoto, SSH, shell arbitraria, caricamento modello o modifiche all’infrastruttura.
Ogni effetto richiede il proprio contratto pubblico e l’ammissione YAI corrente.
Il modello può produrre proposte e spiegazioni; le sue risposte non sono autorità.

## Criteri di verifica

- Identità del target e modello verificabili, senza deduzioni dal nome macchina.
- Una richiesta passa da Studio a Host, Application e provider governato.
- Sono distinguibili stato obbligatorio, Recall/W, omissioni, richiesta serializzata
  e capacità osservata. Un contesto impossibile viene rifiutato prima dell’inferenza.
- L’esecuzione e la risposta hanno riferimenti conservati; una consegna incerta
  non causa un reinvio automatico.
- Riavvio e più client mantengono identità, isolamento e continuità del Case.
- Consegna e rollback richiedono evidenze e una decisione autorizzata; non sono
  dedotti dalla presenza di documenti, da un test fixture o da una risposta fluente.

## Informazioni mancanti

Restano da acquisire osservazioni correnti delle macchine e del servizio,
contratto pubblico di bootstrap/gestione remota, limiti misurati, binding del
provider e risultati del collaudo. Queste assenze sono lavoro aperto, non incidenti
inventati. Il Case deve accumulare tali evidenze senza ricreare la propria storia.
