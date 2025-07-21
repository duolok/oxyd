OXYD - Linux System Monitor 

Projekat za ocenu 10.

## Opis problema

Postojeći alati za praćenje performansi Linux sistema, kao što su htop, top, iotop i slični, pružaju korisnicima informacije o trenutnom stanju sistema. Problem je sto korisnici moraju da koriste nekoliko alata istovremeno da bi imali uvid u celokupan rad sistema.
Međutim, ovi alati imaju nekoliko značajnih ograničenja:

- Nedostatak podrške za naprednu vizualizaciju metrika sistema
- Potreba za korišćenjem više nezavisnih alata kako bi se dobila celovita slika o performansama sistema
- Ograničena mogućnost izvođenja kompleksnih komandi nad procesima u interaktivnom okruženju
- Slaba proširivost i otežana integracija sa spoljnim sistemima za prikupljanje ili eksport podataka

## Cilj projekta

Cilj ovog projekta je razvoj interaktivnog sistemskog monitora za Linux, napisanog u programskom jeziku Rust, koji omogucava:

- Prikaz osnovnih metrika sistema u realnom vremenu (CPU, RAM, disk, mreza, procesi...)
- Interaktivno upravljanje procesima (npr. slanje signala kill, promene prioriteta, suspendovanje i nastavak izvršavanja)
- Sortiranje procesa
- Modularnost i proširivost putem jasno definisanih komponenti
- Vizualizaciju performansi direktno u terminalu, u obliku grafikona, tabela, labela...

## Arhitektura

Neka pocetna arhitektura bi sastojala od nekoliko celina:

- Core Engine - povlaci podatke iz collectora, salje informacije iz tui-a u process manager, sluzi kao HUB za ostale plugin-e
- Process Manager - proverava da li korisnik moze da izvrsi neki proces, dobija signal za interakciju za procesom i PID procesa
- Collectori - u implementaciji bi prikupljali informacije vezne za Linux operativne sisteme, ali bi trebalo da budu prosirivi i na druge
- TUI - ono sto korisnik vidi, vizuelizacija aplikacije, slanje komandi

Arhitektura aplikacije bi trebala da bude takva da se lako mogu dodati nove ekstenzije.

## Buduca unapredjenja

Posto je softver ekstenzibilan, postoji mnogo ideja za buduca unapredjenja. Ideja je da moze lako da se integrise za rad sa drugim aplikacijama. Neka od mogucih unapredjenja:

- cross-platform (mogucnost nadgledanja svih operativnih sistema Windows, Unix, 
- port za nadgledanje vise racunara u istoj mrezi 
- neki oblik server/client arhitekture sa dashboard-om gde je moguce nadgledati i upravljati procesima na vise racunara sa jedne lokacije
- podrska za port na cloud (AWS, Azure, GCP)
- dodatne vizuelizacije
- notifikacije za slanje obavestenja( recimo ako je iskoriscenje procesora vece od nekog procenta)
- monitoring kontejnera (Docker)
- unapredjeni TUI ili Web
- mozda integracija sa Grafanom/Prometheus
- ...
