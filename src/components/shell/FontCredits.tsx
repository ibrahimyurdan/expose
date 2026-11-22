import cinzelLicense from "@/assets/fonts/OFL-Cinzel.txt?raw";
import sourceSansLicense from "@/assets/fonts/OFL-SourceSans3.txt?raw";

// the two bundled fonts' copyright lines, which their license asks to keep
// with them, and both license texts behind a disclosure.
export function FontCredits() {
  return (
    <div className="font-credits">
      <p className="type-body">
        Cinzel, Copyright 2020 The Cinzel Project Authors (https://github.com/NDISCOVER/Cinzel)
      </p>
      <p className="type-body">
        Source Sans 3, © 2023 Adobe (http://www.adobe.com/), with Reserved Font Name ‘Source’
      </p>
      <p className="type-body">Both under the SIL Open Font License 1.1.</p>
      <details className="licenses">
        <summary>Show the font licenses</summary>
        <pre className="license-text type-tiny">{cinzelLicense}</pre>
        <pre className="license-text type-tiny">{sourceSansLicense}</pre>
      </details>
    </div>
  );
}
