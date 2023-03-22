import asyncio
from collections import defaultdict
from statistics import mean
from typing import Optional

import aiohttp
import requests
from fastapi import Depends, FastAPI, HTTPException, Query
from fastapi.responses import HTMLResponse, RedirectResponse
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer

app = FastAPI()

security = HTTPBearer()

WEATHER_TEMPLATE = "<div><h4>{api}:</h4> max - {maxi}, min - {mini}, avg - {avg}</div>"
ACCESS_TOKEN = "HardToCrackToken"


@app.get("/", response_class=HTMLResponse)
def root(
    city: Optional[str] = None, days: Optional[str] = None, prompt: Optional[str] = None
) -> str:
    if city is not None and days is not None:
        response = requests.get(
            "http://127.0.0.1:8000/weather",
            headers={"Authorization": f"Bearer HardToCrackToken"},
            params={"city": city, "days": days},
        )

        if response.ok:
            return response.json()

        return RedirectResponse(url="/?prompt=true")

    return f"""
    <div style="display: flex;justify-content: center;align-items: center;height: 100%;">
        <form>
            {f"<div>Please enter valid city and number of days!</div>" if prompt else ""}
            <div>
                <label for="City">City:</label>
                <input type="text" name="city">
            </div>
            <div>
                <label for="Days">Weather in days:</label>
                <input type="number" min="1" max="5" name="days" value="1">
            </div>
            <input type="submit" value="Search">
        </form>
    </div>
    """


@app.get("/weather")
async def weather(
    days: int = Query(le=5, ge=1),
    city: str = Query(min_length=1),
    credentials: HTTPAuthorizationCredentials = Depends(security),
) -> str:
    if credentials.credentials != ACCESS_TOKEN:
        raise HTTPException(status_code=401)

    location = requests.get(
        f"https://pfa.foreca.com/api/v1/location/search/{city}",
        headers={
            "Authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwOlwvXC9wZmEuZm9yZWNhLmNvbVwvYXV0aG9yaXplXC90b2tlbiIsImlhdCI6MTY3OTE2MTMwNywiZXhwIjo5OTk5OTk5OTk5LCJuYmYiOjE2NzkxNjEzMDcsImp0aSI6IjdhY2U3MWE0ODNlZGM5ZGYiLCJzdWIiOiJjeXByaWFubmV1Z2ViYXVlcjI4IiwiZm10IjoiWERjT2hqQzQwK0FMamxZVHRqYk9pQT09In0.cxwz92Z9-CjgBZS35_7QPDgTMipkCfwGHWxi7pPkfLo"
        },
    )
    if not location.ok or not (location := location.json()["locations"]):
        raise HTTPException(status_code=location.status_code if location else 400)

    lat, lon = location[0]["lat"], location[0]["lon"]

    async with aiohttp.ClientSession() as session:
        futures = [
            _call_weather_api(lat, lon, days, session),
            _call_open_meteo_api(lat, lon, days, session),
            _call_foreca_api(lat, lon, days, session),
        ]
        weather_api, open_meteo, foreca = await asyncio.gather(*futures)

    return f"""
    <div style="display: flex;justify-content: center;align-items: center;height: 100%;">
        <div>
            <h2>Weather in {city} in {days} days:</h2>
            {weather_api}
            {open_meteo}
            {foreca}
        </div>
    </div>
    """


async def _call_weather_api(
    lat: str, lon: str, days: int, session: aiohttp.ClientSession
) -> tuple[dict, str, str]:
    async with session.get(
        "http://api.weatherapi.com/v1/forecast.json",
        params={
            "key": "301879eedf304f2ca91133618231803",
            "q": f"{lat},{lon}",
            "days": days + 1,
        },
    ) as resp:
        if not resp.ok:
            raise HTTPException(status_code=resp.status)

        weather_api_res = await resp.json()

    mini = maxi = avg = None
    for day, forecast in enumerate(weather_api_res["forecast"]["forecastday"]):
        info = forecast["day"]
        if day == days:
            mini, maxi, avg = info["mintemp_c"], info["maxtemp_c"], info["avgtemp_c"]
            break

    return WEATHER_TEMPLATE.format(api="Weather API", maxi=maxi, mini=mini, avg=avg)


async def _call_open_meteo_api(
    lat: str, lon: str, days: int, session: aiohttp.ClientSession
) -> str:
    async with session.get(
        "https://api.open-meteo.com/v1/forecast",
        params={
            "latitude": lat,
            "longitude": lon,
            "hourly": "temperature_2m",
        },
    ) as resp:
        if not resp.ok:
            raise HTTPException(status_code=resp.status)

        open_meteo_res = await resp.json()

    open_meteo_res = open_meteo_res["hourly"]["temperature_2m"]
    whole_day_temp = open_meteo_res[24 * int(days) : 24 * (int(days) + 1)]
    mini, maxi, avg = min(whole_day_temp), max(whole_day_temp), mean(whole_day_temp)

    return WEATHER_TEMPLATE.format(api="Open Meteo API", maxi=maxi, mini=mini, avg=avg)


async def _call_foreca_api(
    lat: str, lon: str, days: int, session: aiohttp.ClientSession
) -> str:
    async with session.get(
        f"https://pfa.foreca.com/api/v1/forecast/3hourly/{lon},{lat}",
        params={"periods": 112},
        headers={
            "Authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwOlwvXC9wZmEuZm9yZWNhLmNvbVwvYXV0aG9yaXplXC90b2tlbiIsImlhdCI6MTY3OTE2MTMwNywiZXhwIjo5OTk5OTk5OTk5LCJuYmYiOjE2NzkxNjEzMDcsImp0aSI6IjdhY2U3MWE0ODNlZGM5ZGYiLCJzdWIiOiJjeXByaWFubmV1Z2ViYXVlcjI4IiwiZm10IjoiWERjT2hqQzQwK0FMamxZVHRqYk9pQT09In0.cxwz92Z9-CjgBZS35_7QPDgTMipkCfwGHWxi7pPkfLo"
        },
    ) as resp:
        if not resp.ok:
            raise HTTPException(status_code=resp.status)

        foreca_res = await resp.json()

    async with session.get(
        f"https://pfa.foreca.com/api/v1/forecast/daily/{lon},{lat}",
        params={"periods": 8},
        headers={
            "Authorization": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJodHRwOlwvXC9wZmEuZm9yZWNhLmNvbVwvYXV0aG9yaXplXC90b2tlbiIsImlhdCI6MTY3OTE2MTMwNywiZXhwIjo5OTk5OTk5OTk5LCJuYmYiOjE2NzkxNjEzMDcsImp0aSI6IjdhY2U3MWE0ODNlZGM5ZGYiLCJzdWIiOiJjeXByaWFubmV1Z2ViYXVlcjI4IiwiZm10IjoiWERjT2hqQzQwK0FMamxZVHRqYk9pQT09In0.cxwz92Z9-CjgBZS35_7QPDgTMipkCfwGHWxi7pPkfLo"
        },
    ) as resp:
        if not resp.ok:
            raise HTTPException(status_code=resp.status)

        foreca_res2 = await resp.json()

    mini = maxi = avg = None
    for day, temp_info in enumerate(foreca_res2["forecast"]):
        if day == days:
            mini, maxi = temp_info["minTemp"], temp_info["maxTemp"]
            break

    temperatures_per_day = defaultdict(list)
    for line in foreca_res["forecast"]:
        day = line["time"][8:10]
        temperatures_per_day[day].append(line["temperature"])

    for day, temperatures in enumerate(temperatures_per_day.values()):
        if day == days:
            avg = mean(temperatures)
            break

    return WEATHER_TEMPLATE.format(api="Foreca API", maxi=maxi, mini=mini, avg=avg)
