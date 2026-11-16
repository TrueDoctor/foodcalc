# Requirements / User Stories
Priorisation with MoSCoW method -> Must, Should, Could, Won't
<!-- Requirements describe the goal state; points already implemented are marked *(umgesetzt)*. -->

## Functional Requirements

### Planing and Scheduling (Before, during and after events)
+ Die App sollte die Möglichkeit bieten, Veranstaltungen zu planen und zu verwalten. (Must)
  + Ein Event umfasst Name, Zeitraum, Budget, Kommentar und Eigentümer (Person oder Gruppe). (Must)
  + Events können als Vorlage geklont werden (Mahlzeiten, Touren, Overrides). (Should) *(umgesetzt)*
+ Die App sollte die Möglichkeit bieten, Rezepte zu verwalten. (Must)
+ Die App sollte die Möglichkeit bieten, Inventare zu verwalten. (Must)

### Energiebasierte Skalierung (Kernkonzept)
Rezepte skalieren nicht beliebig linear — ein Rezept verhält sich bei 300 Personen anders als bei 4.
Foodcalc skaliert deshalb über die Energie:
+ Jede Zutat hat eine Energiedichte (kJ/g). (Must) *(umgesetzt)*
+ Aus den Zutaten ergibt sich die Energiedichte des Rezepts (kJ/g). (Must) *(umgesetzt)*
+ Pro Speise/Mahlzeit kann ein gewünschter Energiebedarf pro Person und die Personenzahl angegeben
  werden; daraus wird die Kochmenge (kg) berechnet, sodass der Energiebedarf der Leute gesichert ist. (Must) *(umgesetzt)*
+ Positionen, die sich nicht über Energie skalieren lassen (z.B. Getränke, Stückgut), werden als
  feste Stück-/Mengen-Ziele geplant statt über Energie. (Could)
+ Die Online-Ansicht zeigt die Basiswerte pro Speise: Energie-Dichte, geplante Energie/Portion,
  Anzahl Portionen, Kosten/Portion. (Should)

*Rezeptverwaltung & Export*
+ Für Events ist es wichtig die skalierten Rezepte als PDF exportieren zu können. (Should)
+ Der User sollte anhand der exportierten Rezepte mit möglichst wenig zusätzlichem Domänenwissen die Speisen zubereiten können. (Should)
  + Die Rezepte sollten klar und verständlich formuliert sein.
  + Mengenangaben sollten auf die geplanten Portionen skaliert sein.
  + Die Rezepte sollten die Zubereitungsschritte in einer sinnvollen Reihenfolge darstellen.
  + Das Rezept sollte alle Subrezepte enthalten, die für die Zubereitung benötigt werden.
  + HACCP-relevante Informationen sollten im Rezept enthalten sein. (Could) ggf. Ankreuzfelder
  + Zusatzinformationen wie z.B. Allergene, vegan, vegetarisch, ... sollten im Rezept enthalten sein. (Could)
+ Für ein Event sollten die Rezepte gesamt exportiert werden können, als auch für jede Speise einzeln. (Should)
+ Für ein Event sollte eine aggregierte Allergenliste als PDF exportiert werden können. (Should)
+ Eine Event-Übersicht (Rahmen, Mahlzeiten, Touren, Vorkochen) sollte als PDF exportiert werden können. (Should) *(umgesetzt)*
+ Beschilderungen (Allergenliste, Speisekarten, Ausgabeschilder (flat | tent)) in verschiedenen Größen für dieses Event können aus Vorlagen erzeugt werden. (Should)
+ Es können Kopien von Rezepten für das Event erstellt werden, um das Rezept für das Event spezifisch anzupassen (Could)

+ Ein User, der ein anderes Rezept sieht, sollte die Möglichkeit haben, dieses Rezept zu kopieren.
  + Dieses kopierte Rezept gehört dann dem User (dessen persönliche Gruppe) und kann von diesem bearbeitet werden. (Should) *(umgesetzt)*

*Rezept-Bearbeitung (Suche & Schnellanlegen)*
+ Beim Zusammenstellen eines Rezepts können Zutaten per Suche gewählt und nach Eigenschaften
  (z.B. nur Gewürze) gefiltert werden; Subrezepte ebenso nach Typ (Mahlzeit/Komponente). (Should) (umgesetzt)
+ Fehlende Zutaten können direkt im Bearbeitungsfluss schnell angelegt werden, ohne die Bearbeitung zu verlassen. (Should) *(umgesetzt)*
+ Löschoperationen weisen auf Verwendungen hin (z.B. „wird in N Rezepten verwendet"), um
  versehentliche Kaskaden zu verhindern. (Should) *(umgesetzt)*
+ Bearbeitung geschieht bewusst und explizit (Bearbeiten-Modi/Drawers), damit nichts versehentlich verändert wird. (Should) *(umgesetzt)*

> Ist die Frage, ob nicht allgemein für Rezepte Copy-On-Write verwendet werden sollte, das macht vieles deutlich einfacher, auch bzgl. der Versionierung. Bedarf darf dann aber einiger Architektonischen Anpassungen? Sollten dann auch in den Events einfach migrierbar sein, oder? Müsste noch überlegt werden.

*Rezept-Aufbau*
+ Rezepte bestehen aus Zutaten, Zubereitungsschritten und ggf. Subrezepten. (Must) *(umgesetzt)*
  + Die App sollte die Möglichkeit bieten, Zutaten zu verwalten. (Must)
  + Die App sollte die Möglichkeit bieten, Zubereitungsschritte zu verwalten. (Must)
  + Die App sollte die Möglichkeit bieten, Subrezepte zu verwalten. (Must)
  + Die App sollte die Möglichkeit bieten, HACCP Kontrollpunkte dem Rezept hinzuzufügen
+ Rezepte können als Mahlzeit oder Komponente klassifiziert werden; beim Event sind beide
  austauschbar einsetzbar (z.B. eine Mahlzeit als Beilage). (Must) *(umgesetzt)*
+ Arbeitsschritte haben eine fixe und eine mengenabhängige Arbeitszeit — Grundlage für die Zeitplanung. (Should) *(umgesetzt)*


*Inventarverwaltung & Einkaufsliste*
Inventare sind Lagerbestände einer Gruppe, die auch für mehrere Veranstaltungen verwendet werden können.
Hierbei ist es wichtig, dass verwendete Zutaten aus den Inventaren abgezogen werden, sodass der aktuelle Bestand immer korrekt ist. Da jedoch bei der Lagerung von Zutaten auch Verluste auftreten können, sollte es möglich sein, den Bestand manuell zu prüfen und anzupassen. (Must)
Hier ist es sinnvoll, dem User Hilfestellungen zu geben.
+ Die App sollte die Möglichkeit bieten, Inventare für das Event zu verwalten. (Must)
+ Die App sollte die Möglichkeit bieten, den aktuellen Bestand der Inventare zu verwalten. (Must)
Für Events sollte es die Möglichkeit geben, private Inventare zu erstellen, die nur für das Event relevant sind, in denen eingetragen werden kann, welche Zutaten für das Event bereits vorhanden sind oder privat zur Verfügung stehen. (Should) Hier sollte es optional auch Kommentare geben, die den Bestand erklären, z.B. von wem die Zutaten stammen oder für was sie verwendet werden sollen.
+ Beim Planen des Einkaufs wird der Bedarf gegen Bestände abgedeckt: Was fehlt, landet auf der Liste;
  die Aufteilung pro Position (z.B. Rest aus dem Vorrat, Hauptmenge vom Großhändler) ist manuell anpassbar. (Should)
+ Buchungen am Inventar werden protokolliert, damit Korrekturen nachvollziehbar bleiben. (Could)
+ Die App sollte die Möglichkeit bieten, eine oder mehrere Einkaufslisten für das Event zu erstellen. (Must)
  + Hierbei sollte eine Heuristik verwendet werden, die die Zutaten den jeweiligen Händlern zuordnet
  + Ein manuelles Anpassen der Einkaufslisten sollte möglich sein.
  + Einkaufslisten sollten sperrbar sein, sodass eine versehentliche Änderung einer gesperrten Liste vermieden wird. Dies ist der Fall, wenn der Einkauf bereits getätigt wurde oder die Bestellung fixiert wurde. (Should)
    Nachkorrekturen sollten dennoch möglich sein, dazu muss die Sperre aufgehoben werden können.
  + Einkaufstouren sollten auch eine Zeit haben, um diese zu planen. (Could)
+ Die App sollte die Möglichkeit bieten, die Einkaufslisten zu exportieren. (Should)

*Zutatenverwaltung*
Zutaten sind die Basis für Rezepte und Inventare.
Jede Zutat hat einen Namen, eine verwertbare Einheit und eine Energiedichte (kJ/g). (Must)
Zutaten können mehrere Eigenschaften tragen (z.B. Gewürz, vegan, Allergen-Gruppe); diese bilden die
Basis für die Ermittlung von Allergenen, vegan, vegetarisch, ... und für Filter in der Suche. (Must)
Für Zutaten können mehrere Händler und deren Preise hinterlegt werden. (Should)
Für den Händler Metro soll der "Metro-Fetch" genutzt werden, um die Preise der Zutaten automatisch zu aktualisieren. Hierfür ist die MetroId bzw. URL zu hinterlegen. (Could)
ggf. muss man hier die Architektur so gestalten, dass unterschiedliche Packungsgrößen der verschiedenen Händler berücksichtigt werden können. (Could)

*Kalkulation*
+ Die App sollte die geplanten Kosten aufschlüsseln: gesamt, je Mahlzeit, je Portion, je Einkaufstour —
  und diese gegen das Event-Budget stellen. (Should)
+ Die Kalkulation basiert auf den geplanten Mengen (inkl. Bestandsabdeckung) und den hinterlegten Preisen. (Should)

*gantt chart*
+ Die App sollte die einzelnen Informationen zu Speisen, Einkaufstouren, Zubereitungen, etc. in einem Gantt-chart anzeigen können. -> Selbe Informationen, nur andere Darstellung. (Must)
  + Die App sollte die Möglichkeit bieten anhand des Gantt-charts die einzelnen Aufgaben zu bearbeiten. (Should)
  + Per Drag&Drop sollte es möglich sein, die Aufgaben zu verschieben und die Zeitplanung zu ändern. (Could)

*Kanban board* (Could) -> Ergänzend zum Gantt-chart, da es eine andere Sichtweise auf die Aufgaben bietet.
+ Die App sollte ein Kanban board für die Planung und Verwaltung der Aufgaben bieten. (Could)
  + Die App sollte die Möglichkeit bieten, die Aufgaben im Kanban board zu bearbeiten. (Could)
  + Anfangs feste Kategorien wie "To Do", "In Progress" und "Done" sollten vorhanden sein. (Could)
  + Priorisierung mit MoSCoW sollte möglich sein. (Could)
  + Per Drag&Drop sollte es möglich sein, die Aufgaben zu verschieben und die Priorisierung zu ändern. (Could)
  + Einzelnen Aufgaben sollten Personen zugewiesen werden können. (Could)

### During events

#### Ordering and Availability (Is not needed for all events, and therefore should be optional and activatable via configuration)
+ Das Statusboard-modul ist pro Event aktivierbar, ist standardmäßig aber deaktiviert. (Should)
+ Die App sollte die Möglichkeit bieten, die Speisen und deren Verfügbarkeit zu verwalten. (Must)
+ Die App sollte die Möglichkeit bieten, die Bestellungen zu verwalten. (Must)
+ Die Ausgabe soll die Möglichkeit haben, nachbestellen zu können. (Should)
+ Es sollte einen Ausgabemodul geben, auf der die aktuellen Speisen aufgezeigt werden. (Should)
  Es sollte Ähnlich wie ein Fahrplan aufgebaut sein, das die aktuellen Speisen und deren Verfügbarkeit anzeigt, sowie die in naher Zukunft verfügbaren Speisen. (Could) -> Externe Ansicht
+ Es sollte eine interne Ansicht geben, die anzeigt, welche Speisen aktuell zubereitet werden sollen und welche Speisen nachbestellt werden müssen. (Could) -> Interne Ansicht
+ Cocktail- und Getränkeplanungen können ebenfalls über die App abgewickelt werden (Bestellung durch Gäste, ggf. Abrechnung pro Person). (ggf. Could -> Not now)

### Login and Security

[//]: # (+ Benutzer sollten sich mit einem Benutzernamen und Passwort anmelden können. &#40;Must&#41;)
+ Die App sollte eine sichere Authentifizierung und Autorisierung bieten. (Must)
+ Benutzerkonten können registriert und von Administratoren verwaltet werden (Sperren, Passwort-Änderung/Reset). (Must) *(umgesetzt)*
+ Sitzungen laufen sicher ab; nicht autorisierte Zugriffe (401) führen zur Anmeldung. (Should) *(umgesetzt)*
+ Der User könnte die Möglichkeit haben sich über externe IDPs mit OAuth2 anzumelden. (Could)
  e.g. PocketId, Keycloak; ggf. bisherige Anmeldung mit Benutzername und Passwort ersetzen.

### Management and Administration
Es gibt Benutzer und Gruppen von Benutzern. Veranstaltungen, Inventare, Rezepte können sowohl einer einzelnen Person als auch einer Gruppe zugeordnet werden.
Anhand der Benutzerrolle und den Berechtigungen kann der Zugriff auf bestimmte Funktionen und Daten eingeschränkt werden.
Das Ziel einer bestimmten Gruppe ist es, dass Berechtigungen und Rollen zentral verwaltet werden können, so dass die Berechtigungen auch für weitere Events übernommen werden können.
Rollen gelten gruppenscoped: Ein Mitglied mit einer Rolle hat in allen Ressourcen, deren Eigentümer die Gruppe ist, dieselben Rechte — z.B. ein "Eventmanager" einer Gruppe hat für alle Veranstaltungen dieser Gruppe dieselben Berechtigungen.
Das Usermanagement sollte später ausbaubar sein und mehrere Gruppen und Benutzerrollen unterstützen. (Could)

+ Die App bietet ein gruppenscopes Rollenmodell: Leser (sehen), Bearbeiter (bearbeiten), Verwalter
  (verwalten zusätzlich Mitglieder und Rollen). (Must) *(umgesetzt)*
+ Die Eigentümerschaft von Ressourcen (Rezepte, Zutaten, Inventare, Events) kann übertragen werden
  (Person->Gruppe, Gruppe->Gruppe, (ggf. auch Gruppe-> Person, Person->Person)); danach gelten für den bisherigen Besitzer die Rechte seiner Rolle
  in der Zielgruppe. (Must) *(umgesetzt)*
+ Administratoren verwalten Benutzerkonten und können sich selbst Gruppenmitgliedschaften geben —
  in Ressourcen sind sie sonst wie jedes andere Mitglied eingeschränkt (kein pauschaler Zugriff). (Must) *(umgesetzt)*
+ Listen (Rezepte, Zutaten, Inventare, Events) lassen sich nach Eigentümer filtern. (Should) *(umgesetzt)*
+ Die App sollte eine Möglichkeit bieten, Benutzerrollen und Berechtigungen zu verwalten. (Must)
+ Die App sollte eine Möglichkeit bieten, die Veranstaltungen zu verwalten. (Must)

### Globale Dokumentenverwaltung (Could), da geteilter Order genauso gut funktionieren würde
Das Ziel der globalen Dokumentenverwaltung ist es, dass bestimmte Dokumente für mehrere Veranstaltungen wichtig sind und zentral an einer Stelle liegen können, damit diese nicht vergessen werden.
e.g. Helfereinweisungen, Sicherheitsunterweisungen, Dokumentationslisten, ... (Should)
+ Die App sollte eine Möglichkeit bieten, globale Dokumente zu verwalten. (Should)
  + Die Dokumente sollten in Kategorien organisiert werden können. (Could)
+ Hier kann man bspw auch wiederverwendbare Beschilderungen ablegen, die für mehrere Veranstaltungen verwendet werden können. (Could)

## Non-Functional Requirements
+ Die Anwendung muss wartbar und erweiterbar sein. (Must)
+ Im Frontend sollte eine funktionsorientierte Strukturierung der Komponenten erfolgen
  (domänenorientierte Pages, geteilte wiederverwendbare Schichten für Patterns/UI). (Must)
+ Die UI ist übersetzbar; UI-Texte liegen zentral in Sprachdateien (Deutsch/Englisch),
  weitere Sprachen sind ohne Code-Änderung ergänzbar. (Must)
+ Die Kernlogik (Skalierung, Mengen-/Energie-/Kosten-Aggregation) ist durch Unit-Tests abgesichert. (Should)
+ Die App bleibt auch bei großen Events (ca. 500 Personen, 7 Tage, >50 Speisen) flüssig und bedienbar. (Should)
+ Kritische Aktionen brauchen klare Bestätigungen; Sperren schützen vor versehentlichen Änderungen
  (z.B. gesperrte Einkaufslisten, Löschen von Positionen (e.g. Events, Zutaten, ...), Übertragung von Eigentum). (Should)
+ Zur Vereinfachung der Navigation werden, im Frontend gezielt Icons eingesetzt, ggf. ergänzend zum Text.

## Won't (bewusst nicht, Stand dieser Fassung)
+ Offline-First/CRDT-Synchronisation — vertagt; eine Server-centric-Anwendung reicht für den Einsatz.
+ Echtzeit-Kollaboration (gemeinsames Editieren mit Live-Sync).
+ Mandantenfähigkeit/SaaS — Einsatz ist self-hosted für Gruppen/Vereine.
