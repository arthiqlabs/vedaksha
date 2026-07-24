"""REST surface tests. Skipped if the optional `rest` extra isn't installed."""
from __future__ import annotations

import pytest

pytest.importorskip("fastapi")
pytest.importorskip("httpx")  # required by fastapi.testclient

from fastapi.testclient import TestClient

from vedaksha.rest import create_app


@pytest.fixture()
def client_noauth() -> TestClient:
    return TestClient(create_app(token=None))


def test_lists_tools(client_noauth: TestClient) -> None:
    r = client_noauth.get("/v1/tools")
    assert r.status_code == 200
    assert len(r.json()) == 15


def test_natal_chart_route(client_noauth: TestClient) -> None:
    r = client_noauth.post(
        "/v1/compute_natal_chart",
        json={"julian_day": 2451545.0, "latitude": 28.6, "longitude": 77.2},
    )
    assert r.status_code == 200
    assert "planets" in r.json()


def test_bad_args_is_422(client_noauth: TestClient) -> None:
    r = client_noauth.post("/v1/compute_natal_chart", json={})
    assert r.status_code == 422


def test_auth_enforced_when_token_set() -> None:
    client = TestClient(create_app(token="sek"))
    body = {"julian_day": 2451545.0, "latitude": 28.6, "longitude": 77.2}
    assert client.post("/v1/compute_natal_chart", json=body).status_code == 401
    ok = client.post(
        "/v1/compute_natal_chart", json=body, headers={"Authorization": "Bearer sek"}
    )
    assert ok.status_code == 200
